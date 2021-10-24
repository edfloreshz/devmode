use anyhow::{Context, Result};
use git2::Repository;
use regex::bytes::Regex;

use crate::home;
use crate::utils::constants::messages::*;
use crate::utils::host::Host;

pub struct Clone {
    pub host: Host,
    pub owner: String,
    pub repos: Vec<String>,
}

impl Default for Clone {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone {
    pub fn new() -> Self {
        Clone {
            host: Host::None,
            owner: String::new(),
            repos: Vec::new(),
        }
    }
    pub fn from(host: Host, owner: String, repos: Vec<String>) -> Self {
        Clone { host, owner, repos }
    }
    pub fn url(&self, index: usize) -> String {
        format!("{}/{}/{}", self.host.url(), self.owner, self.repos.get(index).unwrap())
    }

    pub fn clone_repo(&self) -> Result<()> {
        for (ix, repo) in self.repos.iter().enumerate() {
            let path = format!(
                "{}/Developer/{}/{}/{}",
                home().display(),
                self.host,
                self.owner,
                repo
            );
            println!("Cloning {}/{} from {}...", self.owner, repo, self.host);
            Repository::clone(self.url(ix).as_str(), &path).with_context(|| FAILED_TO_CLONE_REPO)?;
        }
        Ok(())
    }
    pub fn parse_url(url: &str, rx: Regex) -> Result<Clone> {
        let captures = rx.captures(url.as_ref()).unwrap();
        let host = captures
            .get(4)
            .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
            .with_context(|| UNABLE_TO_MAP_URL)?;
        let owner = captures
            .get(6)
            .map(|m| String::from_utf8(Vec::from(m.as_bytes())).unwrap())
            .with_context(|| UNABLE_TO_MAP_URL)?;
        let repo = captures
            .get(7)
            .map(|m| String::from_utf8(Vec::from(m.as_bytes())).unwrap())
            .with_context(|| UNABLE_TO_MAP_URL)?;
        Ok(Clone::from(Host::from(host.into()), owner, vec![repo]))
    }
}
