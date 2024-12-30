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
    #[error("Repository already exists")]
    RepositoryExists,
}
