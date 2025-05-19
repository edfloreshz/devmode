use thiserror::Error;

#[derive(Error, Debug)]
pub enum CloneError {
    #[error("Failed to clone repository")]
    FailedToCloneRepository,
    #[error("This is not a valid Git repository url")]
    InvalidUrl,
    #[error("Path already exists.")]
    PathExists(std::path::PathBuf),
    #[error("{0}")]
    Git(#[from] git2::Error),
    #[error("{0}")]
    GirUrlParseError(#[from] git_url_parse::GitUrlParseError),
}

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Workspace already exists")]
    WorkspaceAlreadyExists,
    #[error("Invalid repository path")]
    InvalidRepoPath,
    #[error("{0}")]
    Io(#[from] std::io::Error),
}
