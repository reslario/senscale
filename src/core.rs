use {
    std::io,
    crate::{
        cfg,
        driver,
        process,
        winutil::current_thread_id,
        msg::{self, ThreadMessage}
    }
};

pub fn run(parent_thread: Option<u32>) -> io::Result<()> {
    let mut config = init(parent_thread)?.config;

    loop {
        if let Some(msg) = msg::peek() {
            match msg {
                msg::Server::Stop => break,
                msg::Server::Reload { msg_thread } => {
                    config = read_config();
                    msg::Client::Printed.send(msg_thread)?
                }
            }
        }

        config
            .entries
            .iter()
            .find(|entry| process::ProcessIds::for_name(&entry.process)
                .any(|proc| process::uses_mouse(proc, entry.only_if_cursor_hidden))
            ).map(|entry| driver::set_sens(entry.sensitivity))
            .unwrap_or_else(|| driver::set_sens(config.default_sensitivity))?;
    }

    Ok(())
}

struct Init {
    config: cfg::Config
}

fn init(parent_thread: Option<u32>) -> io::Result<Init> {
    let config = read_config();

    if let Some(thread) = parent_thread {
        msg::Client::Running { msg_thread: current_thread_id() }
            .send(thread)?;

        msg::Client::Printed.send(thread)?
    }

    Ok(Init { config })
}

fn read_config() -> cfg::Config {
    let config = cfg::read_config()
        .map_err(|e| eprintln!("config error: {}", e))
        .unwrap_or_default();

    eprintln!("default sensitivity = {}", config.default_sensitivity);
    
    if !config.entries.is_empty() {
        eprintln!("scaling for:");

        for entry in &config.entries {
            eprintln!("{}", entry.process)
        }
    }

    config
}
