use {
    state::State,
    std::{
        ptr,
        ffi::OsString,
        path::{PathBuf, Path},
        os::windows::ffi::OsStringExt
    },
    crate::{
        cfg::Config,
        core::driver::Driver
    },
    winapi::{
        shared::{
            minwindef::MAX_PATH,
            windef::{HWND, HWINEVENTHOOK}
        },
        um::{
            handleapi::CloseHandle,
            processthreadsapi::OpenProcess,
            winbase::QueryFullProcessImageNameW,
            winnt::PROCESS_QUERY_LIMITED_INFORMATION,
            winuser::{
                OBJID_CURSOR,
                CHILDID_SELF,
                UnhookWinEvent,
                SetWinEventHook,
                EVENT_OBJECT_HIDE,
                EVENT_OBJECT_SHOW,
                WINEVENT_OUTOFCONTEXT,
                WINEVENT_SKIPOWNPROCESS,
                EVENT_SYSTEM_FOREGROUND,
                GetWindowThreadProcessId
            }
        }
    }
};

mod state;

pub type Handler = fn(&Config, &mut Driver, &Process);

pub struct Hooks {
    focus: HWINEVENTHOOK,
    visibility: HWINEVENTHOOK
}

impl Hooks {
    pub fn set(config: Config, driver: Driver, handler: Handler) -> Option<Hooks> {
        let mut state = State::get();

        if state.is_some() {
            return None
        }

        state.replace(State::new(config, driver, handler));

        let focus = set_hook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, on_focus_changed);
        let visibility = set_hook(EVENT_OBJECT_SHOW, EVENT_OBJECT_HIDE, on_visibility_changed);

        Hooks { focus, visibility }.into()
    }

    pub fn set_config(&mut self, config: Config) {
        if let Some(state) = State::get().as_mut() {
            state.config = config
        }
    }
}

fn set_hook(
    min: u32,
    max: u32,
    handler: unsafe extern "system" fn(HWINEVENTHOOK, u32, HWND, i32, i32, thread: u32, time: u32)
) -> HWINEVENTHOOK {
    unsafe {
        SetWinEventHook(
            min,
            max,
            ptr::null_mut(),
            Some(handler),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS
        )
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        unsafe {
            UnhookWinEvent(self.focus);
            UnhookWinEvent(self.visibility);
            State::get().take();
        }
    }
}

pub struct Process {
    path: PathBuf,
    pub cursor_hidden: Option<bool>
}

impl Process {
    fn new(path: &[u16]) -> Process {
        Process {
            path: OsString::from_wide(path).into(),
            cursor_hidden: None
        }
    }

    pub fn exe(&self) -> &Path {
        &self.path
    }
}

unsafe extern "system" fn on_focus_changed(
    _hook: HWINEVENTHOOK,
    _event: u32,
    window: HWND,
    _object: i32,
    _child: i32,
    _event_thread: u32,
    _event_time: u32
) {
    let mut buf = [0; MAX_PATH];
    let mut proc = 0;
    GetWindowThreadProcessId(window, &mut proc);
    let proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false.into(), proc);
    let mut end = MAX_PATH as _;
    QueryFullProcessImageNameW(proc, 0, buf.as_mut_ptr(), &mut end);
    CloseHandle(proc);

    let process = Process::new(&buf[..end as usize]);
    
    if let Some(state) = State::get().as_mut() {
        state.set_focus(process)
    }
}

unsafe extern "system" fn on_visibility_changed(
    _hook: HWINEVENTHOOK,
    event: u32,
    _window: HWND,
    object: i32,
    child: i32,
    _event_thread: u32,
    _event_time: u32
) {
    if object == OBJID_CURSOR && child == CHILDID_SELF {
        if let Some(state) = State::get().as_mut() {
            state.set_cursor_hidden(event == EVENT_OBJECT_HIDE)
        }
    }
}
