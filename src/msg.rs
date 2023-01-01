use {
    crate::{thread_id::ThreadId, windows::util::validate},
    std::{io, mem::MaybeUninit},
    winapi::{
        shared::windef::HWND,
        um::winuser::{
            DispatchMessageA,
            GetMessageA,
            PostThreadMessageA,
            TranslateMessage,
            MSG,
            WM_COMMAND,
        },
    },
};

const WIN: HWND = -1_isize as _;
const MIN: u32 = 0;
const MAX: u32 = 0;

pub trait ThreadMessage: Sized {
    fn try_from(ident: usize, param: isize) -> Option<Self>;
    fn as_raw(&self) -> (usize, isize);
}

impl ThreadId {
    pub fn send(self, msg: impl ThreadMessage) -> io::Result<()> {
        let (ident, param) = msg.as_raw();
        validate(unsafe { PostThreadMessageA(self.into(), WM_COMMAND, ident, param) })
    }
}

macro_rules! message {
    ($name:ident {
        $($variant:ident $({ $field:ident : $field_type:path })? = $tag:literal),*
    }) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[repr(usize)]
        pub enum $name {
            $(
                $variant $({ $field: $field_type })?
            ),*
        }

        impl ThreadMessage for $name {
            #[allow(non_upper_case_globals)]
            fn try_from(ident: usize, param: isize) -> Option<Self> {
                $(
                    const $variant: usize = $tag as _;
                )*

                match ident {
                    $(
                        $variant => $name::$variant $({ $field: param.into() })?
                    ),*,
                    _ => return None
                }.into()
            }

            fn as_raw(&self) -> (usize, isize) {
                match self {
                    $(
                        // the `- 0` at the end is a hack to provide a default value of
                        // `0` in case no `$field` is present
                        $name::$variant $({ $field })? => ($tag as _, $(isize::from(*$field))? - 0)
                    ),*
                }
            }
        }
    };
}

impl From<isize> for ThreadId {
    fn from(param: isize) -> ThreadId {
        <_>::from(param as u32)
    }
}

impl From<ThreadId> for isize {
    fn from(id: ThreadId) -> isize {
        u32::from(id) as _
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReloadParams {
    pub thread: ThreadId,
    /// will default to `true` when sent if it can't be encoded due to message
    /// param size limitations
    pub print: bool,
}

#[cfg(test)]
static CAN_ENCODE_PRINT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

impl ReloadParams {
    #[cfg(not(test))]
    const fn can_encode_print() -> bool {
        isize::BITS > u32::BITS
    }

    #[cfg(test)]
    fn can_encode_print() -> bool {
        CAN_ENCODE_PRINT.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl From<isize> for ReloadParams {
    fn from(param: isize) -> Self {
        if Self::can_encode_print() && param.is_negative() {
            ReloadParams {
                thread: <_>::from(-param),
                print: false,
            }
        } else {
            ReloadParams {
                thread: <_>::from(param),
                print: true,
            }
        }
    }
}

impl From<ReloadParams> for isize {
    fn from(params: ReloadParams) -> Self {
        if ReloadParams::can_encode_print() && !params.print {
            -isize::from(params.thread)
        } else {
            params.thread.into()
        }
    }
}

message! {
    Server {
        Stop = '🛑',
        Reload { params: ReloadParams } = '♻'
    }
}

message! {
    Client {
        Running { msg_thread: ThreadId } = '🏃',
        Printed = '🖨'
    }
}

pub fn wait<T: ThreadMessage>() -> Option<T> {
    unsafe { read_message(|msg| GetMessageA(msg, WIN, MIN, MAX)) }
}

unsafe fn read_message<T: ThreadMessage>(read: impl Fn(*mut MSG) -> i32) -> Option<T> {
    let mut msg = MaybeUninit::uninit();
    let success = read(msg.as_mut_ptr()) > 0;
    let msg = success.then(|| msg.assume_init())?;

    (msg.message == WM_COMMAND)
        .then(|| T::try_from(msg.wParam, msg.lParam))
        .flatten()
}

pub fn iter<T: ThreadMessage>() -> impl Iterator<Item = T> {
    let mut msg = MaybeUninit::uninit();

    let dispatch = |msg| unsafe {
        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    };

    std::iter::from_fn(move || {
        while unsafe { GetMessageA(msg.as_mut_ptr(), std::ptr::null_mut(), MIN, MAX) > 0 } {
            let msg = unsafe { msg.assume_init() };

            if msg.message == WM_COMMAND {
                match T::try_from(msg.wParam, msg.lParam) {
                    msg @ Some(_) => return msg,
                    None => dispatch(msg),
                }
            } else {
                dispatch(msg)
            }
        }

        None
    })
}

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::windows::thread as win_thread,
        std::{
            sync::{self, mpsc},
            thread,
        },
    };

    #[test]
    fn send_and_receive() {
        let current = win_thread::current_id();
        let client = Client::Running {
            msg_thread: current,
        };
        let server = Server::Reload {
            params: ReloadParams {
                thread: 21_u32.into(),
                print: true,
            },
        };

        let (tx, rx) = mpsc::channel();

        let thread = thread::spawn(move || {
            // force the message queue to be created
            let _ = ThreadId::from(0_u32).send(client);

            tx.send(win_thread::current_id()).unwrap();
            assert_eq!(Some(client), wait());
            current.send(server).unwrap();
        });

        let thread_id = rx.recv().unwrap();
        thread_id.send(client).unwrap();
        thread.join().unwrap();

        assert_eq!(Some(server), wait())
    }

    #[test]
    fn iter() {
        let current = win_thread::current_id();
        let client = [Client::Printed, Client::Running {
            msg_thread: current,
        }];
        let server = [Server::Stop, Server::Reload {
            params: ReloadParams {
                thread: 21_u32.into(),
                print: true,
            },
        }];

        let thread = thread::spawn(move || {
            for msg in client {
                current.send(msg).unwrap();
            }

            for msg in server {
                current.send(msg).unwrap();
            }
        });

        fn assert_received<T, const N: usize>(messages: [T; N])
        where
            T: ThreadMessage + PartialEq + std::fmt::Debug,
        {
            assert_eq!(
                super::iter().take(messages.len()).collect::<Vec<T>>(),
                messages
            )
        }

        assert_received(client);
        assert_received(server);

        thread.join().unwrap();
    }

    #[test]
    fn test_reload_params_repr() {
        assert_eq!(ReloadParams::from(-111111_isize), ReloadParams {
            thread: 111111_u32.into(),
            print: false
        });

        CAN_ENCODE_PRINT.store(false, sync::atomic::Ordering::SeqCst);

        let id = -111111_isize;

        assert_eq!(ReloadParams::from(id), ReloadParams {
            thread: id.into(),
            print: true
        })
    }
}
