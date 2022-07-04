use {
    crate::windows::util::uninit_sized,
    winapi::um::winuser::{GetCursorInfo, CURSORINFO},
};

pub fn hidden() -> bool {
    let mut info = unsafe { uninit_sized::<CURSORINFO>(|i| &mut i.cbSize) };

    let success = unsafe { GetCursorInfo(&mut info) };

    success != 0 && info.flags == 0
}
