//! In-process git operations via `git2` — no shelling out to `git`/`gh`, and
//! no GitHub/GitLab API calls. Credential handling (`credentials`) covers
//! both SSH and HTTPS remotes transparently.

mod credentials;
mod status;
mod url;

pub use status::{CommitSummary, RepoStatus, is_dirty, repo_status};
pub use url::{ParsedUrl, Scheme, parse_url};

use std::path::{Path, PathBuf};

use git2::{Repository, build::RepoBuilder};

use crate::error::{Error, Result};

/// Clones `url` into `dest`, creating parent directories as needed.
pub fn clone(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut builder = RepoBuilder::new();
    builder.fetch_options(credentials::fetch_options());
    builder.clone(url, dest).map_err(|err| {
        // Fixed fallback messages once every credential strategy (SSH
        // agent/key, git credential helper) is exhausted with nothing
        // usable — from git2_credentials itself, or from libgit2's own
        // credential-helper invocation. libgit2 doesn't let us distinguish
        // "wrong credentials" from "repo doesn't exist" here (both look
        // like an auth failure over HTTPS), so give the best actionable
        // guess instead of surfacing either raw string.
        let is_auth_exhausted = matches!(
            err.message(),
            "no valid authentication available"
                | "failed to acquire username/password from local configuration"
        );
        if is_auth_exhausted {
            Error::CloneAuthFailed {
                url: url.to_string(),
            }
        } else {
            Error::Git2(err)
        }
    })?;
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

/// Points the repo at `path`'s `origin` remote at `url`, creating the remote
/// if it doesn't have one yet.
pub fn set_remote_url(path: &Path, url: &str) -> Result<()> {
    let repo = Repository::open(path)?;

    if repo.find_remote("origin").is_ok() {
        repo.remote_set_url("origin", url)?;
    } else {
        repo.remote("origin", url)?;
    }

    Ok(())
}

/// Recursively finds git repositories under `root`, for `dm repo scan`.
/// Doesn't descend into a directory once it's identified as a repo (so
/// nested/vendored repos and submodules aren't reported as separate finds),
/// and skips common large non-repo trees and hidden directories to keep the
/// walk fast.
pub fn find_repos(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk(root, &mut found);
    found
}

const SKIP_DIRS: &[&str] = &["node_modules", "target", ".git"];

fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
    if dir.join(".git").exists() {
        found.push(dir.to_path_buf());
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let is_hidden = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'));
        let is_skipped = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| SKIP_DIRS.contains(&n));
        if is_hidden || is_skipped {
            continue;
        }
        walk(&path, found);
    }
}
