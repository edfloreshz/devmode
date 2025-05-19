use dm_core::error::{CloneError, WorkspaceError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Clone(#[from] CloneError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    CliError(#[from] CliError),
    #[error("{0}")]
    WorkspaceError(#[from] WorkspaceError),
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("{0}")]
    Parse(#[from] clap::Error),
    #[error("Repository already exists. Use a different path or remove the existing directory to continue.")]
    RepositoryExists,
    #[error("Invalid command usage. Run 'dm help' for usage information.")]
    InvalidUsage,
    #[error("Operation cancelled by user.")]
    CancelledByUser,
    #[error("The original repository path could not be found.")]
    OriginalRepositoryPathNotFound,
    #[error("Unknown error occurred. Please report this issue.")]
    Unknown,
}
