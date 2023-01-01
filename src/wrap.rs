use {
    crate::{
        cfg::{self, EditableConfig},
        msg,
        thread_id::ThreadId,
        windows::thread,
        Result,
    },
    std::{
        env,
        fs::{self, File, OpenOptions},
        io::{self, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
        os::windows::process::CommandExt,
        path::{Path, PathBuf},
        process::Command,
    },
    winapi::um::winbase::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS},
};

pub fn run() -> io::Result<()> {
    if instance_file().exists() {
        if already_running()? {
            println!("already running");
            return Ok(())
        } else {
            clean()
        }
    }

    let mut child = Child::spawn()?;
    child.wait_for_output()?;
    child.save()
}

#[derive(Debug, PartialEq, Eq)]
struct Child {
    thread_id: ThreadId,
    output_pos: u64,
}

impl Child {
    fn spawn() -> io::Result<Child> {
        Command::new(env::current_exe()?)
            .arg("run")
            .arg("--foreground")
            .arg("--parent-thread")
            .arg(thread::current_id().to_string())
            .stderr(output_file_write()?)
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .spawn()?;

        let thread_id = if let Some(msg::Client::Running { msg_thread }) = msg::wait() {
            msg_thread
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "child process did not respond",
            ))
        };

        Ok(Child {
            thread_id,
            output_pos: 0,
        })
    }

    fn to_bytes(&self) -> [u8; 12] {
        let mut out = [0; 12];
        let (a, b) = out.split_at_mut(4);
        a.copy_from_slice(&u32::from(self.thread_id).to_le_bytes());
        b.copy_from_slice(&self.output_pos.to_le_bytes());
        out
    }

    fn from_bytes(bytes: [u8; 12]) -> Child {
        let (mut a, mut b) = ([0; 4], [0; 8]);
        let (left, right) = bytes.split_at(4);
        a.copy_from_slice(left);
        b.copy_from_slice(right);

        Child {
            thread_id: u32::from_le_bytes(a).into(),
            output_pos: u64::from_le_bytes(b),
        }
    }

    fn save(&self) -> io::Result<()> {
        let mut file = File::create(instance_file())?;
        file.write_all(&self.to_bytes())
    }

    fn load() -> io::Result<Child> {
        let mut bytes = [0; 12];

        let mut file = File::open(instance_file()).map_err(instance_file_error)?;

        file.read_exact(&mut bytes)?;

        Ok(Child::from_bytes(bytes))
    }

    fn print_output(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        let mut output = output_file_read()?;
        output.seek(SeekFrom::Start(self.output_pos))?;

        self.output_pos += io::copy(&mut output, &mut stdout)?;

        Ok(())
    }

    fn wait_for_output(&mut self) -> io::Result<()> {
        if let Some(msg::Client::Printed { .. }) = msg::wait() {
            self.print_output()
        } else {
            Ok(())
        }
    }

    fn send(&self, msg: msg::Server) -> io::Result<()> {
        self.thread_id.send(msg)
    }

    fn send_msg_with_response(msg: msg::Server) -> io::Result<()> {
        let mut child = Child::load()?;
        child.send(msg)?;
        child.wait_for_output()?;
        child.save()
    }
}

pub fn stop() -> io::Result<()> {
    Child::load()?.send(msg::Server::Stop)?;
    clean();
    Ok(())
}

pub fn reload() -> io::Result<()> {
    Child::send_msg_with_response(msg::Server::Reload {
        params: msg::ReloadParams {
            thread: thread::current_id(),
            print: true,
        },
    })
}

pub fn clean() {
    let _ = (
        fs::remove_file(instance_file()),
        fs::remove_file(output_file_path()),
    );
}

pub fn print_output() -> io::Result<()> {
    let mut child = Child::load()?;
    child.print_output()?;
    child.save()
}

fn instance_file_error(err: io::Error) -> io::Error {
    if err.kind() == io::ErrorKind::NotFound {
        io::Error::new(err.kind(), "no instance running")
    } else {
        err
    }
}

fn instance_file() -> PathBuf {
    env::temp_dir().join("senscale-instance")
}

fn output_file_path() -> PathBuf {
    env::temp_dir().join("senscale-output")
}

fn output_file_write() -> io::Result<File> {
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_file_path())
}

fn output_file_read() -> io::Result<File> {
    File::open(output_file_path())
}

pub fn adjust(process: PathBuf) -> Result {
    let config = cfg::config_dir()?.file();

    for res in io::stdin()
        .lock()
        .lines()
        .filter_map(io::Result::ok)
        .map_while(|line| {
            let line = line.trim();
            (!line.is_empty()).then(|| line.parse::<f64>())
        })
    {
        match res {
            Ok(sens) => set_sens(&config, process.clone(), sens)?,
            Err(e) => eprintln!("failed to parse as number: {e}"),
        }
    }

    Ok(())
}

fn set_sens(config_path: impl AsRef<Path>, process: PathBuf, sens: f64) -> Result {
    let mut config: EditableConfig =
        serde_yaml::from_reader(BufReader::new(File::open(&config_path)?))?;

    config
        .processes
        .entry(process)
        .or_insert_with(<_>::default)
        .sensitivity = sens;

    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(config_path)?;

    cfg::write_config(&config, BufWriter::new(file))?;

    Child::send_msg_with_response(msg::Server::Reload {
        params: msg::ReloadParams {
            thread: thread::current_id(),
            print: false,
        },
    })?;

    Ok(())
}

pub fn config() -> Result {
    let file = cfg::config_dir()?.file();
    if !file.exists() {
        cfg::create_config(&file)?;
    }
    print!("{}", file.display());
    Ok(())
}

fn already_running() -> io::Result<bool> {
    let child = Child::load()?;

    if child.thread_id == thread::current_id() {
        return Ok(false)
    }

    Ok(match thread::process_exe_path(child.thread_id) {
        Some(exe) => exe? == env::current_exe()?,
        None => false,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn child_to_bytes() {
        let thread_id = u32::MAX.into();
        let output_pos = 9259542123273814144;

        assert_eq!(
            [255, 255, 255, 255, 128, 128, 128, 128, 128, 128, 128, 128],
            Child {
                thread_id,
                output_pos
            }
            .to_bytes()
        )
    }

    #[test]
    fn child_from_bytes() {
        assert_eq!(
            Child {
                thread_id: u32::MAX.into(),
                output_pos: 9259542123273814144
            },
            Child::from_bytes([255, 255, 255, 255, 128, 128, 128, 128, 128, 128, 128, 128])
        )
    }

    #[test]
    fn child_roundtrip() {
        let child = Child {
            thread_id: 31267374_u32.into(),
            output_pos: 9613725632561,
        };
        assert_eq!(child, Child::from_bytes(child.to_bytes()))
    }
}
