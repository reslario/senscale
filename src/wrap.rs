use {
    winapi::um::winuser::{WM_COMMAND, PostThreadMessageA},
    crate::{
        Result,
        msg::Message
    },
    std::{
        env,
        path::PathBuf,
        io::{self, Read},
        fs::{self, File},
        process::{Command, Stdio, ChildStdout},
    }
};

pub fn run() -> Result {
    let instance_file = instance_file();

    if instance_file.exists() {
        println!("already running");
        return Ok(())
    }

    let stdout = run_self()?;
    let id = read_thread_id(stdout)?;
    fs::write(instance_file, id.to_le_bytes())?;

    Ok(())
}

fn run_self() -> io::Result<ChildStdout> {
    Command::new(env::current_exe()?)
        .arg("run")
        .arg("--foreground")
        .arg("--print-thread")
        .stdout(Stdio::piped())
        .spawn()
        .map(|child| child.stdout.unwrap())
}

fn read_thread_id(stdout: ChildStdout) -> Result<u32> {
    stdout
        .bytes()
        .filter_map(io::Result::ok)
        .take_while(|b| b.is_ascii_digit())
        .map(char::from)
        .collect::<String>()
        .parse()
        .map_err(<_>::into)
}

pub fn stop() -> io::Result<()> {
    send(Message::Stop)?;
    clean()
}

pub fn reload() -> io::Result<()> {
    send(Message::Reload)
}

pub fn clean() -> io::Result<()> {
    fs::remove_file(instance_file())
}

fn send(msg: Message) -> io::Result<()> {
    let mut thread = 0_u32.to_le_bytes();
    
    File::open(instance_file())
        .map_err(instance_file_error)?
        .read_exact(&mut thread)?;
    
    let thread = u32::from_le_bytes(thread);

    validate(
        unsafe {
            PostThreadMessageA(thread, WM_COMMAND, msg as _, 0)
        }
    )
}

fn instance_file_error(err: io::Error) -> io::Error {
    if err.kind() == io::ErrorKind::NotFound {
        io::Error::new(err.kind(), "no instance running")
    } else {
        err
    }
}

fn validate(result: i32) -> io::Result<()> {
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn instance_file() -> PathBuf {
    env::temp_dir().join("senscale-instance")
}
