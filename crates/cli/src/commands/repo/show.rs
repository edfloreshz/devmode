use dm_core::registry::RegistryStore;

use crate::error::Result;

pub fn run(identifier: String) -> Result<()> {
    let store = RegistryStore::open_default()?;
    let repo = store.find(&identifier)?;

    println!("name:   {}", repo.name);
    println!("path:   {}", repo.path.display());
    println!("remote: {}", repo.remote_url.as_deref().unwrap_or("-"));
    println!("host:   {}", repo.host.as_deref().unwrap_or("-"));
    println!("owner:  {}", repo.owner.as_deref().unwrap_or("-"));
    println!(
        "tags:   {}",
        if repo.tags.is_empty() {
            "-".to_string()
        } else {
            repo.tags.join(", ")
        }
    );
    Ok(())
}
