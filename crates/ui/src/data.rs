//! Loading dm-core state into plain, `Send` snapshots the UI can hold.
//!
//! The stores own a `rusqlite::Connection`, which can't be shared across the
//! worker threads each task runs on — so nothing here holds a store open.
//! Every load opens the stores, reads what it needs, and hands back owned
//! data, exactly as the CLI does per invocation.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

use dm_core::config::Config;
use dm_core::paths;
use dm_core::registry::{RegistryStore, Repo, RepoId};
use dm_core::relayout::{self, Candidate};
use dm_core::workspace::{Workspace, WorkspaceStore};

/// Everything the UI needs to render, loaded in one pass.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub repos: Vec<Repo>,
    pub workspaces: Vec<Workspace>,
    pub config: Config,
    pub drift: Vec<Candidate>,
    /// Workspace ids each repo belongs to, aligned with `repos` by id.
    pub memberships: Vec<(RepoId, Vec<String>)>,
    /// Repos with uncommitted changes, for the list's at-a-glance dot —
    /// checked for every tracked repo up front rather than lazily like
    /// `RepoStatus`, since the list needs an answer for all of them at once.
    pub dirty: HashSet<RepoId>,
}

impl Snapshot {
    pub fn workspaces_for(&self, repo: RepoId) -> &[String] {
        self.memberships
            .iter()
            .find(|(id, _)| *id == repo)
            .map(|(_, names)| names.as_slice())
            .unwrap_or_default()
    }

    pub fn is_dirty(&self, repo: RepoId) -> bool {
        self.dirty.contains(&repo)
    }

    pub fn drift_for(&self, repo: RepoId) -> Option<&Candidate> {
        self.drift.iter().find(|candidate| candidate.id == repo)
    }

    pub fn workspace(&self, id: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|workspace| workspace.id == id)
    }

    pub fn repo(&self, id: RepoId) -> Option<&Repo> {
        self.repos.iter().find(|repo| repo.id == id)
    }
}

/// The members and environment of a single workspace, loaded on demand when
/// one is selected rather than eagerly for every workspace.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceDetail {
    pub id: String,
    pub members: Vec<Repo>,
    pub env: Vec<(String, String)>,
}

pub fn load() -> Result<Snapshot, String> {
    load_inner().map_err(|e| e.to_string())
}

fn load_inner() -> dm_core::Result<Snapshot> {
    let registry = RegistryStore::open_default()?;
    let workspace_store = WorkspaceStore::open_default()?;

    let repos = registry.list(None, None)?;
    let workspaces = workspace_store.list()?;
    let config = Config::load()?;
    let drift = relayout::plan()?;

    let memberships = repos
        .iter()
        .map(|repo| {
            let names = workspace_store
                .workspaces_containing(repo.id)
                .map(|list| list.into_iter().map(|w| w.id).collect())
                .unwrap_or_default();
            (repo.id, names)
        })
        .collect();

    let dirty = repos
        .iter()
        .filter(|repo| dm_core::git::is_dirty(&repo.path))
        .map(|repo| repo.id)
        .collect();

    Ok(Snapshot {
        repos,
        workspaces,
        config,
        drift,
        memberships,
        dirty,
    })
}

/// A repo's git status, loaded on demand when it's selected rather than
/// eagerly for every tracked repo.
pub fn load_repo_status(path: PathBuf) -> Result<dm_core::git::RepoStatus, String> {
    dm_core::git::repo_status(&path).map_err(|e| e.to_string())
}

pub fn load_workspace_detail(id: String) -> Result<WorkspaceDetail, String> {
    load_workspace_detail_inner(&id).map_err(|e| e.to_string())
}

fn load_workspace_detail_inner(id: &str) -> dm_core::Result<WorkspaceDetail> {
    let registry = RegistryStore::open_default()?;
    let workspaces = WorkspaceStore::open_default()?;

    let members = workspaces
        .members(id)?
        .into_iter()
        .map(|repo_id| registry.get(repo_id))
        .collect::<dm_core::Result<Vec<_>>>()?;

    Ok(WorkspaceDetail {
        id: id.to_string(),
        members,
        env: workspaces.env_list(id)?,
    })
}

/// The most recent mtime across devmode's on-disk state.
///
/// Polled by a subscription so the window reflects changes made by `dm` or
/// `dmtui` in another terminal, instead of silently showing stale data.
pub fn state_fingerprint() -> Option<(SystemTime, SystemTime)> {
    let modified = |path: PathBuf| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    };

    let registry = paths::registry_db_file().ok()?;
    let config = paths::config_file().ok()?;

    Some((modified(registry), modified(config)))
}
