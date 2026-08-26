use dm_core::workspace::WorkspaceStore;

use crate::cli::WorkspaceConfigCommand;
use crate::error::Result;

pub fn run(command: WorkspaceConfigCommand) -> Result<()> {
    let store = WorkspaceStore::open_default()?;
    match command {
        WorkspaceConfigCommand::Get { workspace, key } => {
            println!("{}", store.get_config(&workspace, &key)?);
        }
        WorkspaceConfigCommand::Set {
            workspace,
            key,
            value,
        } => store.set_config(&workspace, &key, &value)?,
    }
    Ok(())
}
