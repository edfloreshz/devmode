use std::path::PathBuf;

use dm_core::error::Error as CoreError;
use dm_core::registry::{NewRepo, RegistryStore};

use crate::error::Result;

pub fn run(path: PathBuf, tags: Vec<String>) -> Result<()> {
    let path = path
        .canonicalize()
        .map_err(|_| CoreError::NotADirectory(path.clone()))?;
    if !path.is_dir() {
        return Err(CoreError::NotADirectory(path).into());
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let store = RegistryStore::open_default()?;
    let repo = store.track(NewRepo {
        path,
        name,
        tags,
        ..Default::default()
    })?;
    println!("tracked {} ({})", repo.name, repo.path.display());
    Ok(())
}
