use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no home directory could be determined for this platform")]
    NoHomeDirectory,

    #[error("failed to read config file at {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write config file at {path}: {source}")]
    ConfigWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse config file at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to serialize config: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("unknown config key: {0}")]
    UnknownConfigKey(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("path does not exist or is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("repo is already tracked: {0}")]
    AlreadyTracked(PathBuf),

    #[error("no tracked repo found matching: {0}")]
    RepoNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
