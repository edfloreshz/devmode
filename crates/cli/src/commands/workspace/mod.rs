mod config;
mod create;
mod delete;
mod env;
mod list;
mod membership;
mod show;
mod switch;

use crate::cli::WorkspaceCommand;
use crate::error::Result;

pub fn run(command: WorkspaceCommand) -> Result<()> {
    match command {
        WorkspaceCommand::Create {
            id,
            name,
            description,
            editor,
        } => create::run(id, name, description, editor),
        WorkspaceCommand::Add { workspace, repos } => membership::add(workspace, repos),
        WorkspaceCommand::Remove { workspace, repos } => membership::remove(workspace, repos),
        WorkspaceCommand::List => list::run(),
        WorkspaceCommand::Show { workspace } => show::run(workspace),
        WorkspaceCommand::Config { command } => config::run(command),
        WorkspaceCommand::Env { command } => env::run(command),
        WorkspaceCommand::Switch { workspace, cd } => switch::run(workspace, cd),
        WorkspaceCommand::Delete { workspace, force } => delete::run(workspace, force),
    }
}
