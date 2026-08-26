use dm_core::error::Error as CoreError;
use dm_core::registry::RegistryStore;

use crate::error::Result;
use crate::prompt::confirm;

pub fn run(identifier: String, delete: bool, force: bool) -> Result<()> {
    let store = RegistryStore::open_default()?;
    let repo = store.find(&identifier)?;

    if delete
        && !force
        && !confirm(&format!(
            "delete {} from disk?",
            repo.path.display()
        ))?
    {
        println!("aborted, nothing changed");
        return Ok(());
    }

    if delete {
        std::fs::remove_dir_all(&repo.path).map_err(CoreError::from)?;
    }
    store.remove(repo.id)?;
    println!(
        "untracked {}{}",
        repo.name,
        if delete { " and deleted from disk" } else { "" }
    );
    Ok(())
}
