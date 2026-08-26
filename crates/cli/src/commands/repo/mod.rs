mod list;
mod track;

use crate::cli::RepoCommand;
use crate::error::Result;

pub fn run(command: RepoCommand) -> Result<()> {
    match command {
        RepoCommand::Track { path, tag } => track::run(path, tag),
        RepoCommand::List { tag, host, json } => list::run(tag, host, json),
    }
}
