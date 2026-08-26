#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] dm_core::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
