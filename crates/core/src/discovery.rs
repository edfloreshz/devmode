//! Reconciling the registry with what's actually on disk.
//!
//! Two directions, both explicit and opt-in: `find_untracked` walks a
//! directory tree looking for repos devmode doesn't know about yet, and
//! `check` validates existing entries against the filesystem.
//!
//! The logic lives here rather than in a frontend because `dm repo scan`,
//! `dm repo sync`, and the GUI's Discovery screen all need it, and each
//! frontend should only own how it *presents* the results.

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::git;
use crate::registry::{NewRepo, RegistryStore, Repo};

/// A git repo found on disk that the registry doesn't have an entry for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Discovered {
    pub path: PathBuf,
    pub name: String,
    pub remote_url: Option<String>,
    pub host: Option<String>,
    pub owner: Option<String>,
}

impl From<Discovered> for NewRepo {
    fn from(discovered: Discovered) -> Self {
        NewRepo {
            path: discovered.path,
            name: discovered.name,
            remote_url: discovered.remote_url,
            host: discovered.host,
            owner: discovered.owner,
            tags: Vec::new(),
        }
    }
}

/// Every git repo under `root` that isn't already tracked.
///
/// Reads each one's `origin` remote to fill in host/owner, so a repo tracked
/// from here is immediately eligible for layout checks rather than being
/// invisible to them.
pub fn find_untracked(root: &Path) -> Result<Vec<Discovered>> {
    let store = RegistryStore::open_default()?;
    let mut found = Vec::new();

    for path in git::find_repos(root) {
        if store.find_by_path(&path)?.is_some() {
            continue;
        }

        let remote_url = git::read_origin_url(&path);
        let (host, owner) = match remote_url.as_deref().map(git::parse_url) {
            Some(Ok(parsed)) => (Some(parsed.host), Some(parsed.owner)),
            _ => (None, None),
        };

        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        found.push(Discovered {
            path,
            name,
            remote_url,
            host,
            owner,
        });
    }

    Ok(found)
}

/// Tracks a batch of discovered repos, returning how many were added.
///
/// Skips any that became tracked since discovery rather than failing the
/// whole batch, with a GUI the user may sit on results for a while.
pub fn track_all(discovered: impl IntoIterator<Item = Discovered>) -> Result<usize> {
    let store = RegistryStore::open_default()?;
    let mut tracked = 0;

    for repo in discovered {
        if store.find_by_path(&repo.path)?.is_some() {
            continue;
        }

        store.track(repo.into())?;
        tracked += 1;
    }

    Ok(tracked)
}

/// Something wrong with a tracked repo, found by comparing it against disk.
#[derive(Debug, Clone)]
pub enum Issue {
    /// The recorded path no longer exists.
    Missing { repo: Repo },
    /// The repo's `origin` differs from what the registry recorded.
    RemoteChanged { repo: Repo, current: String },
}

impl Issue {
    pub fn repo(&self) -> &Repo {
        match self {
            Issue::Missing { repo } | Issue::RemoteChanged { repo, .. } => repo,
        }
    }

    /// What resolving this issue will do, for a confirmation prompt or button.
    pub fn resolution(&self) -> String {
        match self {
            Issue::Missing { .. } => "Stop tracking it".to_string(),
            Issue::RemoteChanged { current, .. } => {
                format!("Record the new remote ({current})")
            }
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Issue::Missing { repo } => {
                format!("{} is no longer at {}", repo.name, repo.path.display())
            }
            Issue::RemoteChanged { repo, current } => format!(
                "{}'s remote changed from {} to {current}",
                repo.name,
                repo.remote_url.as_deref().unwrap_or("none"),
            ),
        }
    }
}

/// Validates every tracked repo against the filesystem.
pub fn check() -> Result<Vec<Issue>> {
    let store = RegistryStore::open_default()?;
    let mut issues = Vec::new();

    for repo in store.list(None, None)? {
        if !repo.path.is_dir() {
            issues.push(Issue::Missing { repo });
            continue;
        }

        if let Some(current) = git::read_origin_url(&repo.path)
            && repo.remote_url.as_deref() != Some(current.as_str())
        {
            issues.push(Issue::RemoteChanged { repo, current });
        }
    }

    Ok(issues)
}

/// Applies an issue's resolution: untrack a missing repo, or record a
/// changed remote.
pub fn resolve(issue: &Issue) -> Result<()> {
    let store = RegistryStore::open_default()?;

    match issue {
        Issue::Missing { repo } => store.remove(repo.id),
        Issue::RemoteChanged { repo, current } => store.update_remote(repo.id, current),
    }
}
