//! The repo registry: a local, SQLite-backed index of tracked repos.
//! `repo clone`/`repo create`/`repo track` write entries here; `repo list`
//! reads them back directly instead of walking the filesystem.

mod model;
mod store;

pub use model::{NewRepo, Repo, RepoId};
pub use store::RegistryStore;
