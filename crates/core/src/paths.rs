use std::path::{Component, Path, PathBuf};

use directories::ProjectDirs;

use crate::error::{Error, Result};

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("dev", "devmode", "devmode").ok_or(Error::NoHomeDirectory)
}

/// When `DEVMODE_HOME` is set, all of devmode's state lives under it
/// (`<DEVMODE_HOME>/config`, `<DEVMODE_HOME>/data`) instead of the OS
/// config/data directories. This is the one override that works the same on
/// every platform, which the tests rely on: `directories` keys off the
/// Windows Known Folder API there, so setting `HOME` alone doesn't isolate
/// anything.
fn devmode_home() -> Option<PathBuf> {
    match std::env::var_os("DEVMODE_HOME") {
        Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
        _ => None,
    }
}

/// Directory where devmode stores its SQLite registry database.
pub fn data_dir() -> Result<PathBuf> {
    let dir = match devmode_home() {
        Some(home) => home.join("data"),
        None => project_dirs()?.data_dir().to_path_buf(),
    };
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Directory where devmode stores user-editable configuration (`config.toml`).
pub fn config_dir() -> Result<PathBuf> {
    let dir = match devmode_home() {
        Some(home) => home.join("config"),
        None => project_dirs()?.config_dir().to_path_buf(),
    };
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn registry_db_file() -> Result<PathBuf> {
    Ok(data_dir()?.join("registry.sqlite3"))
}

/// Makes `path` absolute (joining the current directory if it's relative)
/// and lexically cleans up `.`/`..` components, but deliberately does
/// **not** resolve symlinks. devmode always derives repo/workspace paths
/// from one stored root via plain string joining, so as long as every path
/// of record goes through this same normalization, they compare equal by
/// construction, regardless of what any ancestor directory happens to be a
/// symlink to (macOS's `/tmp` -> `/private/tmp`, a repo root pointed at an
/// external drive, a symlinked `$HOME`, etc). Resolving symlinks instead
/// (via `fs::canonicalize`) would require the path to already exist and
/// would make two equivalent-looking paths compare unequal whenever they
/// were canonicalized at different times relative to their directories
/// being created, which is exactly the bug this replaces.
pub fn normalize_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match normalized.components().next_back() {
                Some(Component::Normal(_)) => {
                    normalized.pop();
                }
                _ => normalized.push(component),
            },
            other => normalized.push(other),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an absolute path from `/`-separated parts, with a real root so
    /// the result is absolute on Windows too (`/a` alone isn't).
    fn abs(parts: &str) -> PathBuf {
        let root = if cfg!(windows) { "C:/" } else { "/" };
        PathBuf::from(format!("{root}{parts}"))
    }

    #[test]
    fn normalizes_dot_and_dotdot_components() {
        assert_eq!(normalize_path(&abs("a/b/../c/./d")), abs("a/c/d"));
    }

    #[test]
    fn leaves_clean_absolute_paths_untouched() {
        assert_eq!(normalize_path(&abs("a/b/c")), abs("a/b/c"));
    }

    #[test]
    fn does_not_require_the_path_to_exist() {
        assert_eq!(
            normalize_path(&abs("does/not/exist/../exist")),
            abs("does/not/exist")
        );
    }
}
