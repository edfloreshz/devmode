use std::io::Write;
use std::path::PathBuf;

use dm_core::config::Config;
use dm_core::error::Error as CoreError;
use dm_core::registry::{RegistryStore, Repo};

use crate::error::Result;

struct Move {
    id: dm_core::registry::RepoId,
    name: String,
    from: PathBuf,
    to: PathBuf,
}

pub fn run(apply: bool, yes: bool) -> Result<()> {
    let config = Config::load()?;
    let store = RegistryStore::open_default()?;
    let repos = store.list(None, None)?;

    let moves = plan_moves(&repos, &config);

    if moves.is_empty() {
        println!(
            "all tracked repos already match the current layout ({})",
            config.repo.layout.to_config_string()
        );
        return Ok(());
    }

    println!(
        "{} repo(s) would move to match layout {}:",
        moves.len(),
        config.repo.layout.to_config_string()
    );
    for mv in &moves {
        println!(
            "  {}: {} -> {}",
            mv.name,
            mv.from.display(),
            mv.to.display()
        );
    }

    if !apply {
        println!("\nthis was a preview — re-run with --apply to move these repos and update the registry");
        return Ok(());
    }

    if !yes && !confirm("proceed with moving these repos on disk?")? {
        println!("aborted, nothing was moved");
        return Ok(());
    }

    let mut moved = 0;
    for mv in moves {
        if mv.to.exists() {
            eprintln!(
                "skipping {}: target already exists ({})",
                mv.name,
                mv.to.display()
            );
            continue;
        }
        if let Some(parent) = mv.to.parent() {
            std::fs::create_dir_all(parent).map_err(CoreError::from)?;
        }
        std::fs::rename(&mv.from, &mv.to).map_err(CoreError::from)?;
        store.update_path(mv.id, &mv.to)?;
        moved += 1;
    }
    println!("moved {moved} repo(s)");
    Ok(())
}

fn plan_moves(repos: &[Repo], config: &Config) -> Vec<Move> {
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
            Some(Move {
                id: repo.id,
                name: repo.name.clone(),
                from: repo.path.clone(),
                to: target,
            })
        })
        .collect()
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    std::io::stdout().flush().map_err(CoreError::from)?;
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(CoreError::from)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

/// Whether any tracked repos have host/owner metadata that could drift from
/// the current layout — used to decide whether to suggest `dm repo
/// relayout` after `dm config set layout`.
pub fn has_relayout_candidates() -> Result<bool> {
    let config = Config::load()?;
    let store = RegistryStore::open_default()?;
    let repos = store.list(None, None)?;
    Ok(!plan_moves(&repos, &config).is_empty())
}
