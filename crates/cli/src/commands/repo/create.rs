use std::path::PathBuf;

use dm_core::config::Config;
use dm_core::error::Error as CoreError;
use dm_core::git;
use dm_core::paths;
use dm_core::registry::{NewRepo, RegistryStore};

use crate::error::Result;

pub fn run(name: String, path: Option<PathBuf>, no_git: bool) -> Result<()> {
    let config = Config::load()?;
    let store = RegistryStore::open_default()?;

    // Local repos have no host/owner, so path_layout templates (which key
    // off those) don't apply, they get a flat spot under `local/` instead.
    let dest = path.unwrap_or_else(|| config.repo.root.join("local").join(&name));
    let dest = paths::normalize_path(&dest);

    if dest.exists() {
        return Err(CoreError::DestinationExists(dest).into());
    }

    if no_git {
        std::fs::create_dir_all(&dest).map_err(CoreError::from)?;
    } else {
        git::init(&dest)?;
    }

    let repo = store.track(NewRepo {
        path: dest,
        name,
        ..Default::default()
    })?;
    println!("created {} at {}", repo.name, repo.path.display());
    Ok(())
}
