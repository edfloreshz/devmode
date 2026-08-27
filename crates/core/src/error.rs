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

    #[error("invalid value '{value}' for config key '{key}' (expected {expected})")]
    InvalidConfigValue {
        key: String,
        value: String,
        expected: String,
    },

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

    #[error(
        "invalid path layout: {0} (expected host_owner_repo, owner_repo, flat, or custom:<template>)"
    )]
    InvalidPathLayout(String),

    #[error("cannot relayout {path}: target {target} already exists")]
    RelayoutTargetExists { path: PathBuf, target: PathBuf },

    #[error("git error: {0}")]
    Git2(#[from] git2::Error),

    #[error(
        "could not authenticate with '{url}', the repo may not exist, may be private, \
             or you may need to configure a credential helper or an SSH key in your ssh-agent"
    )]
    CloneAuthFailed { url: String },

    #[error("invalid git url '{url}': {reason}")]
    InvalidGitUrl { url: String, reason: String },

    #[error("destination already exists: {0}")]
    DestinationExists(PathBuf),

    #[error("'{0}' matches more than one tracked repo, try specifying the full path instead")]
    AmbiguousRepo(String),

    #[error("no workspace named '{0}'")]
    WorkspaceNotFound(String),

    #[error("a workspace named '{0}' already exists")]
    WorkspaceAlreadyExists(String),

    #[error("'{repo}' is already in workspace '{workspace}'")]
    AlreadyInWorkspace { workspace: String, repo: String },

    #[error("'{repo}' is not in workspace '{workspace}'")]
    NotInWorkspace { workspace: String, repo: String },
}

pub type Result<T> = std::result::Result<T, Error>;
