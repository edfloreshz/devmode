use dm_core::registry::RegistryStore;

use crate::error::Result;

/// Local-only search over tracked repos, deliberately not "search", to
/// avoid confusion with GitHub-style remote search, which devmode's
/// fully-local design doesn't provide.
pub fn run(query: String) -> Result<()> {
    let store = RegistryStore::open_default()?;
    let query_lower = query.to_lowercase();

    let matches: Vec<_> = store
        .list(None, None)?
        .into_iter()
        .filter(|repo| repo.name.to_lowercase().contains(&query_lower))
        .collect();

    if matches.is_empty() {
        println!("no tracked repos matching '{query}'");
        return Ok(());
    }
    for repo in matches {
        println!("{}\t{}", repo.name, repo.path.display());
    }
    Ok(())
}
