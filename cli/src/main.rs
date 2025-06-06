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
        match e {
            Error::CliError(CliError::RepositoryExists) => {
                log::warning("Try removing the existing directory or use a different path.")
            }
            Error::CliError(CliError::InvalidUsage) => {
                log::warning("Run 'dm help' for usage information.")
            }
            Error::CliError(CliError::CancelledByUser) => log::warning("No changes were made."),
            _ => {}
        }
    }
    Ok(())
}
