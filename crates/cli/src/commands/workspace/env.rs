use dm_core::workspace::WorkspaceStore;

use crate::cli::WorkspaceEnvCommand;
use crate::error::Result;

pub fn run(command: WorkspaceEnvCommand) -> Result<()> {
    let store = WorkspaceStore::open_default()?;
    match command {
        WorkspaceEnvCommand::Set {
            workspace,
            key,
            value,
        } => store.env_set(&workspace, &key, &value)?,
        WorkspaceEnvCommand::Unset { workspace, key } => store.env_unset(&workspace, &key)?,
        WorkspaceEnvCommand::List { workspace } => {
            for (key, value) in store.env_list(&workspace)? {
                println!("{key}={value}");
            }
        }
    }
    Ok(())
}
