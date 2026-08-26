use dm_core::registry::RegistryStore;

use crate::error::Result;

pub fn run(tag: Option<String>, host: Option<String>, json: bool) -> Result<()> {
    let store = RegistryStore::open_default()?;
    let repos = store.list(tag.as_deref(), host.as_deref())?;

    if json {
        println!("{}", serde_json::to_string_pretty(&repos).unwrap());
        return Ok(());
    }

    if repos.is_empty() {
        println!("no repos tracked yet — run `dm repo track <path>` to add one");
        return Ok(());
    }

    for repo in repos {
        let tags = if repo.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", repo.tags.join(", "))
        };
        println!("{}\t{}{}", repo.name, repo.path.display(), tags);
    }
    Ok(())
}
