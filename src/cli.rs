use argh::FromArgs;

/// senscale
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
    Clean(Clean)
}

/// runs senscale
#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
pub struct Run {
    /// runs senscale in the foreground
    #[argh(switch)]
    pub foreground: bool,
    /// prints the main thread id
    /// (used internally when running in the background)
    #[argh(switch)]
    pub print_thread: bool
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
