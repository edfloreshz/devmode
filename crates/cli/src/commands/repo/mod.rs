mod clone;
mod create;
mod find;
mod list;
pub mod relayout;
mod remove;
mod scan;
mod show;
mod sync;
mod track;

use crate::cli::RepoCommand;
use crate::error::Result;

pub fn run(command: RepoCommand) -> Result<()> {
    match command {
        RepoCommand::Clone { url, path } => clone::run(url, path),
        RepoCommand::Create { name, path, no_git } => create::run(name, path, no_git),
        RepoCommand::Show { repo, json } => show::run(repo, json),
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
        RepoCommand::Scan { root, yes } => scan::run(root, yes),
        RepoCommand::Sync { yes } => sync::run(yes),
        RepoCommand::Find { query } => find::run(query),
    }
}
