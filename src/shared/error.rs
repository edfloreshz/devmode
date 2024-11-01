use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Devmode error: {0}")]
    Devmode(#[from] DevmodeError),
    #[error("Argument parsing error: {0}")]
    Parse(#[from] clap::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Utf8 error: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("Requestty error: {0}")]
    Requestty(#[from] requestty::ErrorKind),
    #[error("fs_extra error: {0}")]
    FsExtra(#[from] fs_extra::error::Error),
    #[error("Error: {0}")]
    Generic(&'static str),
    #[error("String error: {0}")]
    String(String),
    #[error("An unknown error ocurred")]
    Unknown,
}

pub fn error<T>(msg: &'static str) -> Result<T, Error> {
    Err(Error::Generic(msg))
}

#[derive(Error, Debug)]
pub enum DevmodeError {
    #[error("No project found.")]
    NoProjectFound,
    #[error("No settings were changed.")]
    NoUrlProvided,
    #[error("Invalid command.")]
    InvalidCommand,
    #[error("The current app options could not be found.\nRun `dm cf --all` to reconfigure them")]
    AppSettingsNotFound,
    #[error("Failed to write settings")]
    FailedToWriteSettings,
    #[error("Failed to parse settings")]
    FailedToParseSettings,
    #[error("Failed to clone repository")]
    FailedToCloneRepository,
    #[error("Failed to set remote repository")]
    FailedToSetRemote,
    #[error("Failed to get branch")]
    FailedToGetBranch,
    #[error("Failed to find workspace")]
    WorkspaceMissing,
    #[error("Please provide a workspace")]
    WorkspaceRequired,
    #[error("Workspace {0} already exists")]
    WorkspaceExists(String),
    #[error("Failed to find project")]
    ProjectNotFound,
    #[error("Multiple projects found. Please specify the project name.")]
    MultipleProjectsFound,
    #[error("Path not found")]
    PathNotFound,
    #[error("File name not found")]
    FileNameNotFound,
}
