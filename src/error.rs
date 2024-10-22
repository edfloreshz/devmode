use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
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
}

pub fn generic<T>(msg: &'static str) -> Result<T, Error> {
    Err(Error::Generic(msg))
}
