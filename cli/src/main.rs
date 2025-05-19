use clap::Parser;
use cli::Cli;
use colored::*;
use devmode::{CliError, Error};

pub mod cli;

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    if let Err(e) = cli.run() {
        eprintln!("{} {}", "error:".red().bold(), e.to_string().red());
        if let Some(suggestion) = get_suggestion(&e) {
            eprintln!("{} {}", "hint:".yellow().bold(), suggestion.yellow());
        }
    }
    Ok(())
}

fn get_suggestion(e: &Error) -> Option<&'static str> {
    match e {
        Error::Cli(CliError::RepositoryExists) => {
            Some("Try removing the existing directory or use a different path.")
        }
        Error::Cli(CliError::InvalidUsage) => Some("Run 'dm help' for usage information."),
        Error::Cli(CliError::CancelledByUser) => Some("No changes were made."),
        _ => None,
    }
}
