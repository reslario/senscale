use {
    crate::winutil::uninit_sized,
    winapi::um::winuser::{CURSORINFO, GetCursorInfo}
};

pub fn hidden() -> bool {
    let mut info = unsafe { 
        uninit_sized::<CURSORINFO>(|i| &mut i.cbSize)
    };

    let success = unsafe { 
        GetCursorInfo(&mut info)
    };

    success != 0 && info.flags == 0
}
