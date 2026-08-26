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

/// Canonicalizes `path`, falling back to canonicalizing the closest existing
/// ancestor and re-appending the rest when `path` itself doesn't exist yet
/// (e.g. a clone root configured before its directory has been created).
/// This still resolves ancestor symlinks (like macOS's `/tmp` ->
/// `/private/tmp`), which a plain "fall back to the raw path" approach would
/// miss — leaving two different-looking paths that actually refer to the
/// same directory, and confusing anything that compares paths by equality
/// (e.g. `dm repo relayout`'s drift detection).
pub fn canonicalize_best_effort(path: &std::path::Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    let mut suffix = Vec::new();
    let mut current = path;
    while let Some(parent) = current.parent() {
        suffix.push(current.file_name().unwrap_or_default().to_os_string());
        if let Ok(canonical_parent) = parent.canonicalize() {
            let mut result = canonical_parent;
            for component in suffix.iter().rev() {
                result.push(component);
            }
            return result;
        }
        current = parent;
    }
    path.to_path_buf()
}
