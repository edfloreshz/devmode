mod clone;
mod create;
mod list;
pub mod relayout;
mod remove;
mod show;
mod track;

use crate::cli::RepoCommand;
use crate::error::Result;

pub fn run(command: RepoCommand) -> Result<()> {
    match command {
        RepoCommand::Clone { url, path } => clone::run(url, path),
        RepoCommand::Create { name, path, no_git } => create::run(name, path, no_git),
        RepoCommand::Show { repo } => show::run(repo),
        RepoCommand::Remove {
            repo,
            delete,
            force,
        } => remove::run(repo, delete, force),
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
