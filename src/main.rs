use anyhow::Result;
use clap::StructOpt;
use cli::Cli;
use colored::Colorize;
mod cli;
mod config;
mod constants;

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Err(e) = cli.run() {
        eprintln!("{} {}", Colorize::red("\nError:"), e);
        if let Some(error) = e.downcast_ref::<git2::Error>() {
            eprintln!("{} {}", Colorize::yellow("Caused by:"), error.message())
        }
    }
    Ok(())
}
