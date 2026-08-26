use dm_core::workspace::WorkspaceStore;

use crate::error::Result;

pub fn run() -> Result<()> {
    let store = WorkspaceStore::open_default()?;
    let workspaces = store.list()?;

    if workspaces.is_empty() {
        println!("no workspaces yet — run `dm workspace create <id>` to add one");
        return Ok(());
    }

    for ws in workspaces {
        let count = store.members(&ws.id)?.len();
        println!("{}\t{} ({count} member(s))", ws.id, ws.name);
    }
    Ok(())
}
