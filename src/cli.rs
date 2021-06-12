use argh::FromArgs;

/// scales your mouse sensitivity on a per-process basis. 
#[derive(FromArgs)]
pub struct Args {
    #[argh(subcommand)]
    pub command: Command
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    Run(Run),
    Stop(Stop),
    Reload(Reload),
    Clean(Clean),
    PrintOutput(PrintOutput)
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
    pub parent_thread: Option<u32>
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
