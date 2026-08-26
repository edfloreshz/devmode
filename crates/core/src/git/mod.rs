//! In-process git operations via `git2` — no shelling out to `git`/`gh`, and
//! no GitHub/GitLab API calls. Credential handling (`credentials`) covers
//! both SSH and HTTPS remotes transparently.

mod credentials;
mod url;

pub use url::{parse_url, ParsedUrl};

use std::path::Path;

use git2::{build::RepoBuilder, Repository};

use crate::error::Result;

/// Clones `url` into `dest`, creating parent directories as needed.
pub fn clone(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut builder = RepoBuilder::new();
    builder.fetch_options(credentials::fetch_options());
    builder.clone(url, dest)?;
    Ok(())
}

/// Initializes a new, empty local git repository at `dest`.
pub fn init(dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    Repository::init(dest)?;
    Ok(())
}

/// Reads the `origin` remote URL of the repo at `path`, if it has one.
pub fn read_origin_url(path: &Path) -> Option<String> {
    let repo = Repository::open(path).ok()?;
    let remote = repo.find_remote("origin").ok()?;
    remote.url().map(str::to_string)
}
