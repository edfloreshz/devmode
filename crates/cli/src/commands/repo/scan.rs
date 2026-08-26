use std::path::PathBuf;

use dm_core::config::Config;
use dm_core::discovery;
use dm_core::paths;
use dm_core::relayout;

use crate::error::Result;
use crate::prompt::confirm;

use super::relayout::print_candidates;

pub fn run(root: Option<PathBuf>, yes: bool) -> Result<()> {
    let config = Config::load()?;
    let root = paths::normalize_path(&root.unwrap_or_else(|| config.repo.root.clone()));

    let mut to_track = Vec::new();
    let mut skipped = 0;

    for found in discovery::find_untracked(&root)? {
        if !yes {
            let label = found.remote_url.as_deref().unwrap_or("no remote");

            if !confirm(&format!("track {} ({label})?", found.path.display()))? {
                skipped += 1;
                continue;
            }
        }

        to_track.push(found);
    }

    let tracked = discovery::track_all(to_track)?;

    println!("tracked {tracked} new repo(s), skipped {skipped}");
    report_layout_drift(yes)
}

/// After scanning, also surface repos that don't match the current layout —
/// not just ones just found, but any already-tracked repo that never
/// matched it — and offer to fix it right here. Honors `scan`'s own --yes
/// so this stays usable non-interactively (inquire's prompts require a TTY).
fn report_layout_drift(yes: bool) -> Result<()> {
    let config = Config::load()?;
    let candidates = relayout::plan()?;
    if candidates.is_empty() {
        return Ok(());
    }

    println!();
    print_candidates(&candidates, &config);
    if yes || confirm("run `dm repo relayout` now to fix this?")? {
        let (moved, skipped) = relayout::apply_candidates(candidates)?;
        for name in &skipped {
            eprintln!("skipping {name}: target already exists");
        }
        println!("moved {moved} repo(s)");
    } else {
        println!("skipped — run `dm repo relayout` any time to fix this later");
    }
    Ok(())
}
