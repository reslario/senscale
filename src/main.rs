#![cfg(windows)]

mod driver;
mod process;
mod cfg;
mod cli;
mod wrap;
mod core;
mod msg;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = run() {
        print!("{}", e)
    }
}

fn run() -> Result {
    let args = argh::from_env::<cli::Args>();

    match args.command {
        cli::Command::Run(args) => if args.foreground {
            core::run(args.print_thread)
        } else {
            wrap::run()
        }?,
        cli::Command::Stop(_) => wrap::stop()?,
        cli::Command::Reload(_) => wrap::reload()?,
        cli::Command::Clean(_) => wrap::clean()?
    };

    Ok(())
}
