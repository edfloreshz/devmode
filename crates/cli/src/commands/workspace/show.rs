use dm_core::registry::{RegistryStore, Repo};
use dm_core::workspace::{Workspace, WorkspaceStore};
use serde::Serialize;

use crate::error::Result;

#[derive(Serialize)]
struct WorkspaceView {
    #[serde(flatten)]
    workspace: Workspace,
    members: Vec<Repo>,
    env: Vec<(String, String)>,
}

pub fn run(workspace: String, json: bool) -> Result<()> {
    let registry = RegistryStore::open_default()?;
    let workspaces = WorkspaceStore::open_default()?;
    let ws = workspaces.get(&workspace)?;
    let members = workspaces
        .members(&ws.id)?
        .into_iter()
        .map(|id| registry.get(id))
        .collect::<dm_core::Result<Vec<_>>>()?;
    let env = workspaces.env_list(&ws.id)?;

    if json {
        let view = WorkspaceView {
            workspace: ws,
            members,
            env,
        };
        println!("{}", serde_json::to_string_pretty(&view).unwrap());
        return Ok(());
    }

    println!("id:          {}", ws.id);
    println!("name:        {}", ws.name);
    println!("description: {}", ws.description.as_deref().unwrap_or("-"));
    println!("editor:      {}", ws.editor.as_deref().unwrap_or("-"));

    println!("members:");
    if members.is_empty() {
        println!("  (none)");
    }
    for repo in &members {
        println!("  {}\t{}", repo.name, repo.path.display());
    }

    if !env.is_empty() {
        println!("env:");
        for (key, value) in &env {
            println!("  {key}={value}");
        }
    }
    Ok(())
}
