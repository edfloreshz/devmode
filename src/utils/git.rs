use dirs::home_dir;
use regex::bytes::Regex;

use crate::models::logic::{Clone, Host};
use crate::Result;

pub fn clone(clone: &Clone) -> Result<()> {
    let path = format!(
        "{}/Developer/{}/{}/{}",
        home_dir().unwrap().display(),
        clone.host.as_ref().unwrap().display(),
        clone.owner.as_ref().unwrap(),
        clone.repo.as_ref().unwrap()
    );
    match git2::Repository::clone(clone.url().as_str(), &path) {
        Ok(_) => {
            println!("{} cloned successfully to {}", clone.repo.unwrap(), path);
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
    Clone::new(Host::from(host), Option::from(owner), Option::from(repo))
}
