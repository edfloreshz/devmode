use std::path::PathBuf;

use directories::ProjectDirs;

use crate::error::{Error, Result};

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("dev", "devmode", "devmode").ok_or(Error::NoHomeDirectory)
}

/// Directory where devmode stores its SQLite registry database.
pub fn data_dir() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    let dir = dirs.data_dir().to_path_buf();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Directory where devmode stores user-editable configuration (`config.toml`).
pub fn config_dir() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    let dir = dirs.config_dir().to_path_buf();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn registry_db_file() -> Result<PathBuf> {
    Ok(data_dir()?.join("registry.sqlite3"))
}
