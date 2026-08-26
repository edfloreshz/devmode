use dm_core::workspace::WorkspaceStore;

use crate::error::Result;
use crate::prompt::confirm;

pub fn run(workspace: String, force: bool) -> Result<()> {
    let store = WorkspaceStore::open_default()?;
    let ws = store.get(&workspace)?;
    let member_count = store.members(&ws.id)?.len();

    if !force
        && member_count > 0
        && !confirm(&format!(
            "delete workspace '{}' with {member_count} member(s)? (repos themselves are not affected)",
            ws.id
        ))?
    {
        println!("aborted, nothing changed");
        return Ok(());
    }

    store.delete(&ws.id)?;
    println!("deleted workspace {}", ws.id);
    Ok(())
}
