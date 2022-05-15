use {
    std::{
        io,
        mem::MaybeUninit
    },
    crate::winutil::validate,
    winapi::{
        shared::windef::HWND,
        um::winuser::{
            MSG,
            WM_COMMAND,
            GetMessageA,
            TranslateMessage,
            DispatchMessageA,
            PostThreadMessageA
        }
    }
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

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(usize)]
pub enum Server {
    Stop,
    Reload { msg_thread: u32 }
}

impl Server {
    const STOP: usize = '🛑' as _;
    const RELOAD: usize = '♻' as _;
}

impl ThreadMessage for Server {
    fn try_from(ident: usize, param: isize) -> Option<Self> {
        match ident {
            Server::STOP => Server::Stop,
            Server::RELOAD => Server::Reload { msg_thread: param as _ },
            _ => return None
        }.into()
    }

    fn as_raw(&self) -> (usize, isize) {
        match self {
            Server::Stop => (Server::STOP, 0),
            Server::Reload { msg_thread } => (Server::RELOAD, *msg_thread as _)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(usize)]
pub enum Client {
    Running { msg_thread: u32 },
    Printed
}

impl Client {
    const RUNNING: usize = '🏃' as _;
    const PRINTED: usize = '🖨' as _;
}

impl ThreadMessage for Client {
    fn try_from(ident: usize, param: isize) -> Option<Self> {
        match ident {
            Client::RUNNING => Client::Running { msg_thread: param as _ },
            Client::PRINTED => Client::Printed,
            _ => return None
        }.into()
    }

    fn as_raw(&self) -> (usize, isize) {
        match self {
            Client::Running { msg_thread: main_thread } => (Client::RUNNING, *main_thread as _),
            Client::Printed => (Client::PRINTED, 0),
        }
    }
}

fn send_message(thread: u32, ident: usize, param: isize) -> io::Result<()> {
    validate(
        unsafe {
            PostThreadMessageA(thread, WM_COMMAND, ident, param)
        }
    )
}

pub fn wait<T: ThreadMessage>() -> Option<T> {
    unsafe {
        read_message(|msg| GetMessageA(msg, WIN, MIN, MAX))
    }
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
                    None => dispatch(msg)
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
        crate::winutil::current_thread_id,
        std::{
            thread,
            sync::mpsc
        },
    };

    #[test]
    fn send_and_receive() {
        let current = current_thread_id();
        let client = Client::Running { msg_thread: current };
        let server = Server::Reload { msg_thread: 21 };

        let (tx, rx) = mpsc::channel();

        let thread = thread::spawn(move || {
            tx.send(current_thread_id()).unwrap();
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
        let current = current_thread_id();
        let client = [
            Client::Printed,
            Client::Running { msg_thread: current }
        ];
        let server = [
            Server::Stop,
            Server::Reload { msg_thread: 21 }
        ];

        let thread = thread::spawn(move || {
            for msg in client {
                msg.send(current).unwrap();
            }

            for msg in server {
                msg.send(current).unwrap();
            }
        });

        fn assert_received<T, const N: usize>(messages: [T; N])
        where T: ThreadMessage + PartialEq + std::fmt::Debug {
            assert_eq!(super::iter().take(messages.len()).collect::<Vec<T>>(), messages)
        }

        assert_received(client);
        assert_received(server);

        thread.join().unwrap();
    }
}
