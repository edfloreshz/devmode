use std::path::PathBuf;

use dm_core::config::Config;
use dm_core::error::Error as CoreError;
use dm_core::git;
use dm_core::paths;
use dm_core::registry::{NewRepo, RegistryStore};

use crate::error::Result;

pub fn run(url: String, path: Option<PathBuf>) -> Result<()> {
    let parsed = git::parse_url(&url)?;
    let config = Config::load()?;
    let store = RegistryStore::open_default()?;

    let dest = match path {
        Some(path) => path,
        None => config.repo.root.join(config.repo.layout.render(
            &parsed.host,
            &parsed.owner,
            &parsed.name,
        )),
    };
    let dest = paths::normalize_path(&dest);

    if dest.exists() {
        return Err(CoreError::DestinationExists(dest).into());
    }

    git::clone(&url, &dest)?;

    let repo = store.track(NewRepo {
        path: dest,
        name: parsed.name,
        remote_url: Some(url),
        host: Some(parsed.host),
        owner: Some(parsed.owner),
        tags: Vec::new(),
    })?;
    println!("cloned {} into {}", repo.name, repo.path.display());
    Ok(())
}
