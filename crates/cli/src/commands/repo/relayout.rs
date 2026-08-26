use dm_core::config::Config;
use dm_core::relayout::{self, Candidate};

use crate::error::Result;
use crate::prompt::confirm;

pub fn run(apply: bool, yes: bool) -> Result<()> {
    let config = Config::load()?;
    let candidates = relayout::plan()?;

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

    let (moved, skipped) = relayout::apply_candidates(candidates)?;
    for name in &skipped {
        eprintln!("skipping {name}: target already exists");
    }
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

/// Whether any tracked repos have host/owner metadata that could drift from
/// the current layout — used to decide whether to suggest `dm repo
/// relayout` after `dm config set layout`.
pub fn has_relayout_candidates() -> Result<bool> {
    Ok(relayout::has_candidates()?)
}
