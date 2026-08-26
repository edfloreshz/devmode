use dm_core::git;
use dm_core::registry::RegistryStore;

use crate::error::Result;
use crate::prompt::confirm;

pub fn run(yes: bool) -> Result<()> {
    let store = RegistryStore::open_default()?;
    let mut ok = 0;

    for repo in store.list(None, None)? {
        if !repo.path.is_dir() {
            println!("missing: {} ({})", repo.name, repo.path.display());
            if yes || confirm("  untrack it?")? {
                store.remove(repo.id)?;
                println!("  untracked");
            }
            continue;
        }

        if let Some(current_url) = git::read_origin_url(&repo.path) {
            if repo.remote_url.as_deref() != Some(current_url.as_str()) {
                println!(
                    "remote changed: {} ({} -> {current_url})",
                    repo.name,
                    repo.remote_url.as_deref().unwrap_or("-"),
                );
                if yes || confirm("  update recorded remote?")? {
                    store.update_remote(repo.id, &current_url)?;
                    println!("  updated");
                }
                continue;
            }
        }

        ok += 1;
    }

    println!("{ok} repo(s) ok");
    Ok(())
}
