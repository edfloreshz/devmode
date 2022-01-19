use anyhow::Result;
use clap::StructOpt;
use colored::Colorize;

use cli::Cli;

mod cli;
mod config;
mod constants;

fn main() -> Result<()> {
    let cli = Cli::parse();
    handle_errors(cli.run());
    Ok(())
}

fn handle_errors(cli: Result<()>) {
    if let Err(e) = cli {
        eprintln!("{} {}", Colorize::red("\nError:"), e);
        if let Some(error) = e.downcast_ref::<git2::Error>() {
            eprintln!("{} {}", Colorize::yellow("Caused by:"), error.message())
        }
    }
}
