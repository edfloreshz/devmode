use dm_core::error::Error as CoreError;
use dm_core::registry::RegistryStore;
use dm_core::workspace::WorkspaceStore;

use crate::error::Result;
use crate::prompt::confirm;
use crate::resolve::resolve_repo;

pub fn run(identifier: String, delete: bool, force: bool) -> Result<()> {
    let store = RegistryStore::open_default()?;
    let repo = resolve_repo(&store, &identifier)?;

    let workspaces = WorkspaceStore::open_default()?;
    let member_of = workspaces.workspaces_containing(repo.id)?;
    if !member_of.is_empty() && !force {
        let names = member_of
            .iter()
            .map(|w| w.id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if !confirm(&format!(
            "{} is in {} workspace(s) ({names}) — untrack it anyway? (it will be removed from them too)",
            repo.name,
            member_of.len()
        ))? {
            println!("aborted, nothing changed");
            return Ok(());
        }
    }

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
