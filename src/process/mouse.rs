use winapi::{
    um::winuser::{
        CURSORINFO,
        EnumWindows,
        GetCursorInfo,
        GUITHREADINFO,
        GetGUIThreadInfo,
        GetWindowThreadProcessId
    },
    shared::{
        windef::HWND,
        minwindef::{BOOL, TRUE, FALSE, LPARAM}
    }
};

struct FindState {
    process_id: u32,
    check_hidden: bool,
    found: bool
}

pub fn uses_mouse(process_id: u32, check_hidden: bool) -> bool {
    let mut state = FindState {
        process_id,
        check_hidden,
        found: false
    };

    unsafe { EnumWindows(Some(find_mouse_user), &mut state as *mut _ as _) };

    state.found
}

unsafe extern "system" fn find_mouse_user(handle: HWND, state: LPARAM) -> BOOL {
    let mut state = &mut *(state as *mut FindState);

    let mut process_id = 0;
    let thread_id = GetWindowThreadProcessId(handle, &mut process_id);

    if process_id == state.process_id
        && thread_uses_mouse(thread_id)
        && (!state.check_hidden || mouse_hidden()) 
    {
        state.found = true;
        FALSE
    } else {
        TRUE
    }
}

fn thread_uses_mouse(thread: u32) -> bool {
    let mut info = unsafe { 
        new_info::<GUITHREADINFO>(|i| &mut i.cbSize)
    };

    let success = unsafe {
        GetGUIThreadInfo(thread, &mut info)
    };
    
    success != 0 && (!info.hwndCapture.is_null() || !info.hwndFocus.is_null())
}

fn mouse_hidden() -> bool {
    let mut info = unsafe { 
        new_info::<CURSORINFO>(|i| &mut i.cbSize)
    };

    let success = unsafe { 
        GetCursorInfo(&mut info)
    };

    success != 0 && info.flags == 0
}

#[allow(clippy::uninit_assumed_init)]
unsafe fn new_info<T>(size_member: fn(&mut T) -> &mut u32) -> T {
    use std::mem::{MaybeUninit, size_of};

    let mut info = MaybeUninit::<T>::uninit().assume_init();
    *size_member(&mut info) = size_of::<T>() as _;
    info
}
