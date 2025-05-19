use clap::Parser;
use cli::Cli;
use error::{CliError, Error};

pub mod cli;
pub mod error;
pub mod helpers;
pub mod log;

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    if let Err(e) = cli.run() {
        log::error(&e.to_string());
        if let Some(suggestion) = get_suggestion(&e) {
            log::warning(suggestion);
        }
    }
    Ok(())
}

fn get_suggestion(e: &Error) -> Option<&'static str> {
    match e {
        Error::CliError(CliError::RepositoryExists) => {
            Some("Try removing the existing directory or use a different path.")
        }
        Error::CliError(CliError::InvalidUsage) => Some("Run 'dm help' for usage information."),
        Error::CliError(CliError::CancelledByUser) => Some("No changes were made."),
        _ => None,
    }
}
