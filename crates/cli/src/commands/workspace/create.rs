use dm_core::workspace::{NewWorkspace, WorkspaceStore};

use crate::error::Result;

pub fn run(
    id: String,
    name: Option<String>,
    description: Option<String>,
    editor: Option<String>,
) -> Result<()> {
    let store = WorkspaceStore::open_default()?;
    let name = name.unwrap_or_else(|| id.clone());
    let ws = store.create(NewWorkspace {
        id,
        name,
        description,
        editor,
    })?;
    println!("created workspace {}", ws.id);
    Ok(())
}
