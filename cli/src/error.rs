use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Services(#[from] crate::services::Error),
    #[error("{0}")]
    Parse(#[from] clap::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Cli(#[from] CliError),
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Repository already exists. Use a different path or remove the existing directory to continue.")]
    RepositoryExists,
    #[error("Invalid command usage. Run 'dm help' for usage information.")]
    InvalidUsage,
    #[error("Operation cancelled by user.")]
    CancelledByUser,
    #[error("Unknown error occurred. Please report this issue.")]
    Unknown,
}
