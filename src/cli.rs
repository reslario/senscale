use std::str::FromStr;

use {crate::thread_id::ThreadId, argh::FromArgs, std::path::PathBuf};

/// scales your mouse sensitivity on a per-process basis.
#[derive(FromArgs)]
pub struct Args {
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    Run(Run),
    Stop(Stop),
    Reload(Reload),
    Clean(Clean),
    PrintOutput(PrintOutput),
    Adjust(Adjust),
    Config(Config),
}

/// runs senscale
#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
pub struct Run {
    /// runs senscale in the foreground
    #[argh(switch)]
    pub foreground: bool,
    /// sets the thread used for message passing
    /// (used internally when running in the background)
    #[argh(option)]
    pub parent_thread: Option<ThreadId>,
}

/// stops senscale
#[derive(FromArgs)]
#[argh(subcommand, name = "stop")]
pub struct Stop {}

/// reloads the config file
#[derive(FromArgs)]
#[argh(subcommand, name = "reload")]
pub struct Reload {}

/// cleans up any state left over by improper termination
#[derive(FromArgs)]
#[argh(subcommand, name = "clean")]
pub struct Clean {}

/// prints all the output from a running instance that
/// has accumulated since the last time it was checked
#[derive(FromArgs)]
#[argh(subcommand, name = "print-output")]
pub struct PrintOutput {}

/// lets you easily adjust a process' sensitivity until you've found the right
/// one
#[derive(FromArgs)]
#[argh(subcommand, name = "adjust")]
pub struct Adjust {
    /// the .exe of the process
    #[argh(positional)]
    pub process: PathBuf,
}

/// prints the config file path and creates the file if it doesn't exist
#[derive(FromArgs)]
#[argh(subcommand, name = "config")]
pub struct Config {}

impl FromStr for ThreadId {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(u32::into)
    }
}
