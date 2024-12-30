use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    GirUrlParseError(#[from] git_url_parse::GitUrlParseError),
    #[error("{0}")]
    Git(#[from] git2::Error),
    #[error("{0}")]
    Clone(#[from] CloneError),
}

#[derive(Error, Debug)]
pub enum CloneError {
    #[error("Failed to clone repository")]
    FailedToCloneRepository,
    #[error("This is not a valid Git repository url")]
    InvalidUrl,
    #[error("Path already exists.")]
    PathExists(std::path::PathBuf),
}
