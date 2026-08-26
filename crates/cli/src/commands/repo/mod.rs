mod list;
pub mod relayout;
mod track;

use crate::cli::RepoCommand;
use crate::error::Result;

pub fn run(command: RepoCommand) -> Result<()> {
    match command {
        RepoCommand::Track {
            path,
            tag,
            host,
            owner,
        } => track::run(path, tag, host, owner),
        RepoCommand::List { tag, host, json } => list::run(tag, host, json),
        RepoCommand::Relayout { apply, yes } => relayout::run(apply, yes),
    }
}
