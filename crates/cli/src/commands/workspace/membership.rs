use dm_core::registry::RegistryStore;
use dm_core::workspace::WorkspaceStore;

use crate::error::Result;
use crate::resolve::resolve_repo;

pub fn add(workspace: String, repos: Vec<String>) -> Result<()> {
    let registry = RegistryStore::open_default()?;
    let workspaces = WorkspaceStore::open_default()?;
    for identifier in repos {
        let repo = resolve_repo(&registry, &identifier)?;
        // Report by the repo's name rather than letting the store's
        // AlreadyInWorkspace error surface, it only knows the repo id.
        match workspaces.add_member(&workspace, repo.id) {
            Ok(()) => println!("added {} to {workspace}", repo.name),
            Err(dm_core::Error::AlreadyInWorkspace { .. }) => {
                println!("{} is already in {workspace}", repo.name);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

pub fn remove(workspace: String, repos: Vec<String>) -> Result<()> {
    let registry = RegistryStore::open_default()?;
    let workspaces = WorkspaceStore::open_default()?;
    for identifier in repos {
        let repo = resolve_repo(&registry, &identifier)?;
        match workspaces.remove_member(&workspace, repo.id) {
            Ok(()) => println!("removed {} from {workspace}", repo.name),
            Err(dm_core::Error::NotInWorkspace { .. }) => {
                println!("{} is not in {workspace}", repo.name);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
