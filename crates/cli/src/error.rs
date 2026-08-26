#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] dm_core::Error),

    #[error("prompt error: {0}")]
    Prompt(#[from] inquire::InquireError),
}

pub type Result<T> = std::result::Result<T, Error>;
