use {
    crate::{
        cfg::{self, Config},
        msg,
        thread_id::ThreadId,
        windows::thread,
    },
    driver::Driver,
    hook::Hooks,
    std::{io, path::Path},
};

mod cursor;
mod driver;
mod hook;

pub fn run(parent_thread: Option<ThreadId>) -> io::Result<()> {
    let Init { config, driver } = match parent_thread {
        Some(thread) => {
            thread.send(msg::Client::Running {
                msg_thread: thread::current_id(),
            })?;

            match init() {
                Ok(init) => {
                    thread.send(msg::Client::Printed)?;
                    init
                }
                Err(e) => {
                    eprint!("initialization error: {e}");
                    return thread.send(msg::Client::Printed)
                }
            }
        }
        None => init()?,
    };

    let mut hook = Hooks::set(config, driver, on_focus_changed).expect("hooks already set");

    for msg in msg::iter() {
        match msg {
            msg::Server::Stop => break,
            msg::Server::Reload { params } => {
                let config = read_config();

                if params.print {
                    print_config(&config)
                }

                hook.set_config(config);
                params.thread.send(msg::Client::Printed)?
            }
        }
    }

    Ok(())
}

struct Init {
    config: cfg::Config,
    driver: Driver,
}

fn init() -> io::Result<Init> {
    let config = read_config();
    print_config(&config);
    let driver = Driver::new()?;

    Ok(Init { config, driver })
}

fn read_config() -> Config {
    cfg::read_config()
        .map_err(|e| eprintln!("config error: {e}"))
        .unwrap_or_default()
}

fn print_config(config: &Config) {
    eprintln!("default sensitivity = {}", config.default_sensitivity);

    if !config.processes.is_empty() {
        eprintln!("scaling for:");

        for (process, entry) in &config.processes {
            eprintln!("{} ({})", process.display(), entry.sensitivity)
        }
    }
}

fn on_focus_changed(config: &Config, driver: &mut Driver, process: &hook::Process) {
    let res = config
        .processes
        .get(process.exe())
        .or_else(|| config.processes.get(Path::new(process.exe().file_name()?)))
        .filter(|entry| entry.cursor_matches(process))
        .map(|entry| driver.set_sens(entry.sensitivity))
        .unwrap_or_else(|| driver.set_sens(config.default_sensitivity));

    if let Err(e) = res {
        eprint!("{e}")
    }
}

impl cfg::Entry {
    fn cursor_matches(&self, process: &hook::Process) -> bool {
        !self.only_if_cursor_hidden || process.cursor_hidden.unwrap_or_else(cursor::hidden)
    }
}
