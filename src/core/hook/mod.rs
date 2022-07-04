use {
    crate::{cfg::Config, core::driver::Driver, windows},
    state::State,
    std::{
        path::{Path, PathBuf},
        ptr,
    },
    winapi::{
        shared::windef::{HWINEVENTHOOK, HWND},
        um::winuser::{
            GetWindowThreadProcessId,
            SetWinEventHook,
            UnhookWinEvent,
            CHILDID_SELF,
            EVENT_OBJECT_HIDE,
            EVENT_OBJECT_SHOW,
            EVENT_SYSTEM_FOREGROUND,
            OBJID_CURSOR,
            WINEVENT_OUTOFCONTEXT,
            WINEVENT_SKIPOWNPROCESS,
        },
    },
};

mod state;

pub type Handler = fn(&Config, &mut Driver, &Process);

pub struct Hooks {
    focus: HWINEVENTHOOK,
    visibility: HWINEVENTHOOK,
}

impl Hooks {
    pub fn set(config: Config, driver: Driver, handler: Handler) -> Option<Hooks> {
        let mut state = State::get();

        if state.is_some() {
            return None
        }

        state.replace(State::new(config, driver, handler));

        let focus = set_hook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            on_focus_changed,
        );
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
    handler: unsafe extern "system" fn(HWINEVENTHOOK, u32, HWND, i32, i32, thread: u32, time: u32),
) -> HWINEVENTHOOK {
    unsafe {
        SetWinEventHook(
            min,
            max,
            ptr::null_mut(),
            Some(handler),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        unsafe {
            UnhookWinEvent(self.focus);
            UnhookWinEvent(self.visibility);
        }

        State::get().take();
    }
}

pub struct Process {
    path: PathBuf,
    pub cursor_hidden: Option<bool>,
}

impl Process {
    fn new(path: PathBuf) -> Process {
        Process {
            path,
            cursor_hidden: None,
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
    _event_time: u32,
) {
    let mut proc = 0;
    GetWindowThreadProcessId(window, &mut proc);
    let path = windows::process::exe_path(proc).unwrap_or_default();

    let process = Process::new(path);

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
    _event_time: u32,
) {
    if object == OBJID_CURSOR && child == CHILDID_SELF {
        if let Some(state) = State::get().as_mut() {
            state.set_cursor_hidden(event == EVENT_OBJECT_HIDE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hooks_lifecycle() {
        let set_hooks = || Hooks::set(<_>::default(), Driver::dummy().unwrap(), |_, _, _| {});

        let hooks = set_hooks().unwrap();

        assert!(!hooks.focus.is_null());
        assert!(!hooks.visibility.is_null());

        assert!(set_hooks().is_none());

        drop(hooks);

        assert!(State::get().is_none())
    }
}
