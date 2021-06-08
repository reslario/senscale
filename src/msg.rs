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
            PM_REMOVE,
            WM_COMMAND,
            GetMessageA,
            PeekMessageA,
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

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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

pub fn peek<T: ThreadMessage>() -> Option<T> {
    unsafe {
        read_message(|msg| PeekMessageA(msg, WIN, MIN, MAX, PM_REMOVE))
    }
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
