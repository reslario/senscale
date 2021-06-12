use {
    winapi::um::winbase::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS},
    crate::{
        winutil::current_thread_id,
        msg::{self, ThreadMessage}
    },
    std::{
        env,
        path::PathBuf,
        process::Command,
        fs::{self, File},
        os::windows::process::CommandExt,
        io::{self, Read, Write, Seek, SeekFrom}
    }
};

pub fn run() -> io::Result<()> {
    if instance_file().exists() {
        println!("already running");
        return Ok(())
    }

    let mut child = Child::spawn()?;
    child.wait_for_output()?;
    child.save()
}

struct Child {
    thread_id: u32,
    output_pos: u64
}

impl Child {
    fn spawn() -> io::Result<Child> {
        Command::new(env::current_exe()?)
            .arg("run")
            .arg("--foreground")
            .arg("--parent-thread")
            .arg(current_thread_id().to_string())
            .stderr(output_file_write()?)
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .spawn()?;

        let thread_id = if let Some(msg::Client::Running { msg_thread }) = msg::wait() {
            msg_thread
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "child process did not respond"
            ))
        };

        Ok(Child {
            thread_id,
            output_pos: 0
        })
    }

    fn save(&self) -> io::Result<()> {
        let mut file = File::create(instance_file())?;
        file.write_all(&self.thread_id.to_le_bytes())?;
        file.write_all(&self.output_pos.to_le_bytes())
    }

    fn load() -> io::Result<Child> {
        let mut thread = 0_u32.to_le_bytes();
        let mut pos = 0_u64.to_le_bytes();

        let mut file = File::open(instance_file())
            .map_err(instance_file_error)?;

        file.read_exact(&mut thread)?;
        file.read_exact(&mut pos)?;

        Ok(Child {
            thread_id: u32::from_le_bytes(thread),
            output_pos: u64::from_le_bytes(pos)
        })
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
        msg.send(self.thread_id)
    }
}



pub fn stop() -> io::Result<()> {
    Child::load()?.send(msg::Server::Stop)?;
    clean();
    Ok(())
}

pub fn reload() -> io::Result<()> {
    let mut child = Child::load()?;
    child.send(msg::Server::Reload { msg_thread: current_thread_id() })?;
    child.wait_for_output()?;
    child.save()
}

pub fn clean() {
    let _ = (
        fs::remove_file(instance_file()),
        fs::remove_file(output_file_path())
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
