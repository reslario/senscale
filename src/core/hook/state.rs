use {
    once_cell::sync::Lazy,
    std::sync::{Mutex, MutexGuard},
    crate::{
        cfg::Config,
        core::{
            driver::Driver,
            hook::{Process, Handler}
        }
    }
};

pub struct State {
    pub config: Config,
    driver: Driver,
    handler: Handler,
    focus: Option<Process>
}

impl State {
    pub fn new(config: Config, driver: Driver, handler: Handler) -> State {
        State {
            config,
            driver,
            handler,
            focus: None
        }
    }

    pub fn get() -> MutexGuard<'static, Option<State>> {
        STATE.lock().unwrap()
    }

    pub fn set_focus(&mut self, process: Process) {
        self.focus.replace(process);
        self.call_handler()
    }

    pub fn set_cursor_hidden(&mut self, hidden: bool) {
        if let Some(proc) = &mut self.focus {
            proc.cursor_hidden.replace(hidden);
            self.call_handler()
        }
    }

    fn call_handler(&mut self) {
        if let Some(proc) = &self.focus {
            (self.handler)(&self.config, &mut self.driver, proc);
        }
    }
}

static STATE: Lazy<Mutex<Option<State>>> = Lazy::new(<_>::default);
