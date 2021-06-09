use {
    crate::{
        process::Match,
        winutil::uninit_sized
    },
    winapi::{
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
            minwindef::{BOOL, LPARAM}
        }
    }
};

struct FindState<'a> {
    processes: &'a [Match<'a>],
    found: Option<Match<'a>>
}

pub fn mouse_user<'a>(processes: &'a [Match<'a>]) -> Option<Match<'a>> {
    let mut state = FindState {
        processes,
        found: None
    };

    enum_windows(find_mouse_user, &mut state);

    state.found
}

fn find_mouse_user(handle: HWND, state: &mut FindState) -> bool {
    let mut process_id = 0;
    let thread_id = unsafe {
        GetWindowThreadProcessId(handle, &mut process_id)
    };

    state.found = state
        .processes
        .iter()
        .copied()
        .find(|proc| is_mouse_user(*proc, process_id, thread_id));

    state.found.is_none()
}

fn is_mouse_user(process: Match, process_id: u32, thread_id: u32) -> bool {
    process_id == process.process_id
        && thread_uses_mouse(thread_id)
        && (!process.entry.only_if_cursor_hidden || mouse_hidden())
}

fn thread_uses_mouse(thread: u32) -> bool {
    let mut info = unsafe { 
        uninit_sized::<GUITHREADINFO>(|i| &mut i.cbSize)
    };

    let success = unsafe {
        GetGUIThreadInfo(thread, &mut info)
    };
    
    success != 0 && (!info.hwndCapture.is_null() || !info.hwndFocus.is_null())
}

fn mouse_hidden() -> bool {
    let mut info = unsafe { 
        uninit_sized::<CURSORINFO>(|i| &mut i.cbSize)
    };

    let success = unsafe { 
        GetCursorInfo(&mut info)
    };

    success != 0 && info.flags == 0
}

fn enum_windows<T, F>(f: F, state: &mut T)
where F: FnMut(HWND, &mut T) -> bool {
    struct State<'a, T, F> {
        f: F,
        data: &'a mut T
    }

    unsafe extern "system" fn callback<T, F>(handle: HWND, state: LPARAM) -> BOOL
    where F: FnMut(HWND, &mut T) -> bool {
        let state = &mut *(state as *mut State<T, F>);

        (state.f)(handle, state.data).into()
    }

    let mut state = State { 
        f,
        data: state
    };
    
    unsafe { EnumWindows(Some(callback::<T, F>), &mut state as *mut _ as _) };
}
