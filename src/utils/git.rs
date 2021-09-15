use {
    dirs::home_dir,
    regex::bytes::Regex,
    crate::models::logic::{Clone, Host},
    crate::Result,
};


pub fn clone(clone: &Clone) -> Result<()> {
    let path = format!(
        "{}/Developer/{}/{}/{}",
        home_dir().unwrap().display(),
        clone.host.as_ref().unwrap(),
        clone.owner.as_ref().unwrap(),
        clone.repo.as_ref().unwrap()
    );
    println!(
        "Cloning {}/{} from {}...",
        clone.owner.as_ref().unwrap(),
        clone.repo.as_ref().unwrap(),
        clone.host.as_ref().unwrap().to_string()
    );
    match git2::Repository::clone(clone.url().as_str(), &path) {
        Ok(_) => {
            println!(
                "{} cloned successfully to {}",
                clone.repo.as_ref().unwrap(),
                path
            );
            Ok(())
        }
        Err(e) => Err(Box::new(e)),
    }
}

pub fn parse_url(url: &str, rx: Regex) -> Clone {
    let captures = rx.captures(url.as_ref()).unwrap();
    let host = captures
        .get(4)
        .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
        .unwrap();
    let owner = captures
        .get(6)
        .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
        .unwrap();
    let repo = captures
        .get(7)
        .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
        .unwrap();
    Clone::new(
        Host::from(host.into()),
        Option::from(owner.to_string()),
        Option::from(repo.to_string()),
    )
}
