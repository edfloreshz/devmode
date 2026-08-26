use std::path::PathBuf;

use dm_core::config::Config;
use dm_core::error::Error as CoreError;
use dm_core::registry::{RegistryStore, Repo, RepoId};

use crate::error::Result;
use crate::prompt::confirm;

pub struct Candidate {
    pub id: RepoId,
    pub name: String,
    pub from: PathBuf,
    pub to: PathBuf,
}

pub fn run(apply: bool, yes: bool) -> Result<()> {
    let config = Config::load()?;
    let candidates = plan()?;

    if candidates.is_empty() {
        println!(
            "all tracked repos already match the current layout ({})",
            config.repo.layout.to_config_string()
        );
        return Ok(());
    }

    print_candidates(&candidates, &config);

    if !apply {
        println!("\nthis was a preview — re-run with --apply to move these repos and update the registry");
        return Ok(());
    }

    if !yes && !confirm("proceed with moving these repos on disk?")? {
        println!("aborted, nothing was moved");
        return Ok(());
    }

    let moved = apply_candidates(candidates)?;
    println!("moved {moved} repo(s)");
    Ok(())
}

pub fn print_candidates(candidates: &[Candidate], config: &Config) {
    println!(
        "{} repo(s) would move to match layout {}:",
        candidates.len(),
        config.repo.layout.to_config_string()
    );
    for c in candidates {
        println!("  {}: {} -> {}", c.name, c.from.display(), c.to.display());
    }
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
/// (and warning about) any target that already exists. Returns how many
/// were actually moved.
pub fn apply_candidates(candidates: Vec<Candidate>) -> Result<usize> {
    let store = RegistryStore::open_default()?;
    let mut moved = 0;
    for c in candidates {
        if c.to.exists() {
            eprintln!("skipping {}: target already exists ({})", c.name, c.to.display());
            continue;
        }
        if let Some(parent) = c.to.parent() {
            std::fs::create_dir_all(parent).map_err(CoreError::from)?;
        }
        std::fs::rename(&c.from, &c.to).map_err(CoreError::from)?;
        store.update_path(c.id, &c.to)?;
        moved += 1;
    }
    Ok(moved)
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
pub fn has_relayout_candidates() -> Result<bool> {
    Ok(!plan()?.is_empty())
}
