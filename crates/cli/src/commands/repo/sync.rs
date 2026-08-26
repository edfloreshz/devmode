use dm_core::discovery::{self, Issue};
use dm_core::registry::RegistryStore;

use crate::error::Result;
use crate::prompt::confirm;

pub fn run(yes: bool) -> Result<()> {
    let tracked = RegistryStore::open_default()?.list(None, None)?.len();
    let issues = discovery::check()?;

    // Counted before resolving, since untracking a missing repo would
    // otherwise shrink the total it's measured against.
    let ok = tracked.saturating_sub(issues.len());

    for issue in &issues {
        match issue {
            Issue::Missing { repo } => {
                println!("missing: {} ({})", repo.name, repo.path.display());

                if yes || confirm("  untrack it?")? {
                    discovery::resolve(issue)?;
                    println!("  untracked");
                }
            }
            Issue::RemoteChanged { repo, current } => {
                println!(
                    "remote changed: {} ({} -> {current})",
                    repo.name,
                    repo.remote_url.as_deref().unwrap_or("-"),
                );

                if yes || confirm("  update recorded remote?")? {
                    discovery::resolve(issue)?;
                    println!("  updated");
                }
            }
        }
    }

    println!("{ok} repo(s) ok");
    Ok(())
}
