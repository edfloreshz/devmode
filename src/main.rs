use anyhow::Result;
use clap::StructOpt;
use colored::Colorize;

// use crate::config::settings::Settings;
use cli::Cli;
// use once_cell::sync::OnceCell;

mod cli;
mod config;
mod constants;

// static SETTINGS: OnceCell<Settings> = OnceCell::new();

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
