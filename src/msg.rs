use {
    crate::windows::util::validate,
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

    fn send(&self, thread: u32) -> io::Result<()> {
        let (ident, param) = self.as_raw();
        send_message(thread, ident, param)
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
                        $variant => $name::$variant $({ $field: param as _ })?
                    ),*,
                    _ => return None
                }.into()
            }

            fn as_raw(&self) -> (usize, isize) {
                match self {
                    $(
                        // the `- 0` at the end is a hack to provide a default value of
                        // `0` in case no `$field` is present
                        $name::$variant $({ $field })? => ($tag as _, $(*$field as isize)? - 0)
                    ),*
                }
            }
        }
    };
}

message! {
    Server {
        Stop = '🛑',
        Reload { msg_thread: u32 } = '♻'
    }
}

message! {
    Client {
        Running { msg_thread: u32 } = '🏃',
        Printed = '🖨'
    }
}

fn send_message(thread: u32, ident: usize, param: isize) -> io::Result<()> {
    validate(unsafe { PostThreadMessageA(thread, WM_COMMAND, ident, param) })
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
        std::{sync::mpsc, thread},
    };

    #[test]
    fn send_and_receive() {
        let current = win_thread::current_id();
        let client = Client::Running {
            msg_thread: current,
        };
        let server = Server::Reload { msg_thread: 21 };

        let (tx, rx) = mpsc::channel();

        let thread = thread::spawn(move || {
            tx.send(win_thread::current_id()).unwrap();
            assert_eq!(Some(client), wait());
            server.send(current).unwrap();
        });

        let thread_id = rx.recv().unwrap();
        client.send(thread_id).unwrap();
        thread.join().unwrap();

        assert_eq!(Some(server), wait())
    }

    #[test]
    fn iter() {
        let current = win_thread::current_id();
        let client = [Client::Printed, Client::Running {
            msg_thread: current,
        }];
        let server = [Server::Stop, Server::Reload { msg_thread: 21 }];

        let thread = thread::spawn(move || {
            for msg in client {
                msg.send(current).unwrap();
            }

            for msg in server {
                msg.send(current).unwrap();
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
}
