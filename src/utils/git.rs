use {
    crate::models::logic::{Clone, Host},
    crate::Result,
    anyhow::Context,
    dirs::home_dir,
    git2::Repository,
    regex::bytes::Regex,
};

pub fn clone(clone: &Clone) -> Result<()> {
    let path = format!(
        "{}/Developer/{}/{}/{}",
        home_dir().unwrap().display(),
        clone.host.unwrap(),
        clone.owner.as_ref().unwrap(),
        clone.repo.as_ref().unwrap()
    );
    println!(
        "Cloning {}/{} from {}...",
        clone.owner.as_ref().unwrap(),
        clone.repo.as_ref().unwrap(),
        clone.host.unwrap()
    );
    Repository::clone(clone.url().as_str(), &path)
        .with_context(|| format!("Failed to clone repository."))?;
    Ok(())
}

pub fn parse_url(url: &str, rx: Regex) -> Result<Clone> {
    let captures = rx.captures(url.as_ref()).unwrap();
    let host = captures
        .get(4)
        .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
        .with_context(|| "Could not map url.")?;
    let owner = captures
        .get(6)
        .map(|m| String::from_utf8(Vec::from(m.as_bytes())).unwrap())
        .with_context(|| "Could not map url.")?;
    let repo = captures
        .get(7)
        .map(|m| String::from_utf8(Vec::from(m.as_bytes())).unwrap())
        .with_context(|| "Could not map url.")?;
    Ok(
        Clone::new(
            Host::from(host.into()),
            Option::from(owner),
            Option::from(repo),
        )
    )
}
