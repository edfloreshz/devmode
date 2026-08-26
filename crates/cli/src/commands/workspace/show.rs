use dm_core::registry::RegistryStore;
use dm_core::workspace::WorkspaceStore;

use crate::error::Result;

pub fn run(workspace: String) -> Result<()> {
    let registry = RegistryStore::open_default()?;
    let workspaces = WorkspaceStore::open_default()?;
    let ws = workspaces.get(&workspace)?;

    println!("id:          {}", ws.id);
    println!("name:        {}", ws.name);
    println!("description: {}", ws.description.as_deref().unwrap_or("-"));
    println!("editor:      {}", ws.editor.as_deref().unwrap_or("-"));

    println!("members:");
    let member_ids = workspaces.members(&ws.id)?;
    if member_ids.is_empty() {
        println!("  (none)");
    }
    for id in member_ids {
        let repo = registry.get(id)?;
        println!("  {}\t{}", repo.name, repo.path.display());
    }

    let env = workspaces.env_list(&ws.id)?;
    if !env.is_empty() {
        println!("env:");
        for (key, value) in env {
            println!("  {key}={value}");
        }
    }
    Ok(())
}
