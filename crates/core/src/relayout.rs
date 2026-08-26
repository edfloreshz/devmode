//! Computing and applying moves that bring tracked repos in line with the
//! configured `repo.layout` — shared by `dm repo relayout`, `dm repo scan`'s
//! drift report, and (eventually) the TUI's drift indicator.

use std::path::PathBuf;

use crate::config::Config;
use crate::error::Result;
use crate::registry::{RegistryStore, Repo, RepoId};

pub struct Candidate {
    pub id: RepoId,
    pub name: String,
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Computes which tracked repos (with known host/owner) don't match the
/// current layout, and where they'd move to.
pub fn plan() -> Result<Vec<Candidate>> {
    let config = Config::load()?;
    let store = RegistryStore::open_default()?;
    let repos = store.list(None, None)?;
    Ok(plan_moves(&repos, &config))
}

/// Moves each candidate on disk and updates its registry entry, skipping
/// any target that already exists rather than overwriting it. Returns
/// `(moved, skipped)` — skipped entries are just names, for the caller to
/// report however it likes (println, TUI status line, etc).
pub fn apply_candidates(candidates: Vec<Candidate>) -> Result<(usize, Vec<String>)> {
    let store = RegistryStore::open_default()?;
    let mut moved = 0;
    let mut skipped = Vec::new();
    for c in candidates {
        if c.to.exists() {
            skipped.push(c.name);
            continue;
        }
        if let Some(parent) = c.to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&c.from, &c.to)?;
        store.update_path(c.id, &c.to)?;
        moved += 1;
    }
    Ok((moved, skipped))
}

fn plan_moves(repos: &[Repo], config: &Config) -> Vec<Candidate> {
    repos
        .iter()
        .filter_map(|repo| {
            let host = repo.host.as_deref()?;
            let owner = repo.owner.as_deref()?;
            let target = config
                .repo
                .root
                .join(config.repo.layout.render(host, owner, &repo.name));
            if target == repo.path {
                return None;
            }
            Some(Candidate {
                id: repo.id,
                name: repo.name.clone(),
                from: repo.path.clone(),
                to: target,
            })
        })
        .collect()
}

/// Whether any tracked repos have host/owner metadata that could drift from
/// the current layout — used to decide whether to suggest `dm repo
/// relayout` after `dm config set layout`.
pub fn has_candidates() -> Result<bool> {
    Ok(!plan()?.is_empty())
}
