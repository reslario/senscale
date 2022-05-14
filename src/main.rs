#![cfg(windows)]

mod cfg;
mod cli;
mod wrap;
mod core;
mod msg;
mod winutil;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = run() {
        eprint!("error: {}", e)
    }
}

fn run() -> Result {
    let args = argh::from_env::<cli::Args>();

    match args.command {
        cli::Command::Run(args) => if args.foreground {
            core::run(args.parent_thread)
        } else {
            wrap::run()
        }?,
        cli::Command::Stop(_) => wrap::stop()?,
        cli::Command::Reload(_) => wrap::reload()?,
        cli::Command::Clean(_) => wrap::clean(),
        cli::Command::PrintOutput(_) => wrap::print_output()?,
        cli::Command::Adjust(args) => wrap::adjust(args.process)?
    };

    Ok(())
}
