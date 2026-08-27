use dm_core::error::Error as CoreError;
use dm_core::registry::{RegistryStore, Repo};

use crate::error::Result;
use crate::prompt::interactive;

/// Resolves a `repo` argument, prompting the user to pick one via `inquire`
/// when the name matches more than one tracked repo instead of erroring.
/// When `interactive` config is disabled, an ambiguous match errors instead
/// (there's no non-TTY way to offer a picker), matching the pre-`inquire`
/// behavior.
pub fn resolve_repo(store: &RegistryStore, identifier: &str) -> Result<Repo> {
    let mut matches = store.find_matches(identifier)?;
    match matches.len() {
        0 => Err(CoreError::RepoNotFound(identifier.to_string()).into()),
        1 => Ok(matches.remove(0)),
        _ if !interactive() => Err(CoreError::AmbiguousRepo(identifier.to_string()).into()),
        _ => {
            let options: Vec<String> = matches
                .iter()
                .map(|r| format!("{} ({})", r.name, r.path.display()))
                .collect();
            let selected = inquire::Select::new(
                &format!("multiple repos match '{identifier}', pick one:"),
                options,
            )
            .prompt()?;
            let index = matches
                .iter()
                .position(|r| format!("{} ({})", r.name, r.path.display()) == selected)
                .expect("selected option came from the same list");
            Ok(matches.remove(index))
        }
    }
}
