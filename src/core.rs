use {
    std::{
        mem::MaybeUninit,
        convert::TryInto
    },
    winapi::um::{
        processthreadsapi::GetCurrentThreadId,
        winuser::{
            PM_REMOVE,
            WM_COMMAND,
            PeekMessageA
        }
    },
    crate::{
        cfg,
        driver,
        process,
        Result,
        msg::Message
    }
};

pub fn run(print_thread: bool) -> Result {
    if print_thread {
        println!("{}", unsafe { GetCurrentThreadId() });
    }

    let mut config = read_config();

    loop {
        if let Some(msg) = read_message() {
            match msg {
                Message::Stop => break,
                Message::Reload => config = read_config()
            }
        }

        config
            .entries
            .iter()
            .find(|entry| process::ProcessIds::for_name(&entry.process)
                .any(|proc| process::uses_mouse(proc, entry.only_if_cursor_hidden))
            ).map(|entry| driver::set_sens(entry.sensitivity))
            .unwrap_or_else(driver::reset)?;
    }

    Ok(())
}

fn read_config() -> cfg::Config {
    let config = cfg::read_config()
        .map_err(|e| eprintln!("config error: {}", e))
        .unwrap_or_default();
    
    if !config.entries.is_empty() {
        eprintln!("scaling for:");

        for entry in &config.entries {
            eprintln!("{}", entry.process)
        }
    }

    config
}

fn read_message() -> Option<Message> {
    let mut msg = MaybeUninit::uninit();

    let msg = unsafe {
        let available = PeekMessageA(
            msg.as_mut_ptr(),
            -1_isize as _,
            0,
            0,
            PM_REMOVE
        ) != 0;

        available.then(|| msg.assume_init())
    }?;

    (msg.message == WM_COMMAND)
        .then(|| msg.wParam.try_into().ok())
        .flatten()
}
