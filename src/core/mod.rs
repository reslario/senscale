use {
    hook::Hooks,
    driver::Driver,
    std::{
        io,
        path::Path
    },
    crate::{
        cfg::{self, Config},
        winutil::current_thread_id,
        msg::{self, ThreadMessage}
    }
};

mod hook;
mod cursor;
mod driver;

pub fn run(parent_thread: Option<u32>) -> io::Result<()> {
    let Init { config, driver } = match parent_thread {
        Some(thread) => {
            msg::Client::Running { msg_thread: current_thread_id() }
                .send(thread)?;

            match init() {
                Ok(init) => {
                    msg::Client::Printed.send(thread)?;
                    init
                },
                Err(e) => {
                    eprint!("{}", e);
                    return msg::Client::Printed.send(thread)
                }
            }
        },
        None => init()?
    };

    let mut hook = Hooks::set(config, driver, on_focus_changed)
        .expect("hooks already set");

    for msg in msg::iter() {
        match msg {
            msg::Server::Stop => break,
            msg::Server::Reload { msg_thread } => {
                hook.set_config(read_config());
                msg::Client::Printed.send(msg_thread)?
            }
        }
    }

    Ok(())
}

struct Init {
    config: cfg::Config,
    driver: Driver
}

fn init() -> io::Result<Init> {
    let config = read_config();
    let driver = Driver::new()?;

    Ok(Init { config, driver })
}

fn read_config() -> Config {
    let config = cfg::read_config()
        .map_err(|e| eprintln!("config error: {}", e))
        .unwrap_or_default();

    eprintln!("default sensitivity = {}", config.default_sensitivity);
    
    if !config.entries.is_empty() {
        eprintln!("scaling for:");

        for entry in &config.entries {
            eprintln!("{} ({})", entry.process, entry.sensitivity)
        }
    }

    config
}

fn on_focus_changed(config: &Config, driver: &mut Driver, process: &hook::Process) {
    let res = config
        .entries
        .iter()
        .find(|entry| entry.exe_matches(process))
        .filter(|entry| entry.cursor_matches(process))
        .map(|entry| driver.set_sens(entry.sensitivity))
        .unwrap_or_else(|| driver.set_sens(config.default_sensitivity));

    if let Err(e) = res {
        eprint!("{}", e)
    }
}

impl cfg::Entry {
    fn exe_matches(&self, process: &hook::Process) -> bool {
        if is_path(&self.process) {
            process.exe() == self.process.as_str()
        } else {
            Path::new(process.exe())
                .file_name()
                .map(|name| name == self.process.as_str())
                .unwrap_or_default()
        }
    }

    fn cursor_matches(&self, process: &hook::Process) -> bool {
        !self.only_if_cursor_hidden 
            || process.cursor_hidden.unwrap_or_else(cursor::hidden)
    }
}

fn is_path(exe: impl AsRef<Path>) -> bool {
    exe.as_ref().parent() != Some("".as_ref())
}
