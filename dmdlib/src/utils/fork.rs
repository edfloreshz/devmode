use anyhow::{Context, Result};
use git2::Repository;
use regex::bytes::Regex;
use std::path::Path;

use crate::home;
use crate::utils::constants::messages::*;
use crate::utils::host::Host;

pub struct Fork {
    pub host: Host,
    pub upstream: String,
    pub owner: String,
    pub repo: String,
    pub repo_path: String,
}

impl Default for Fork {
    fn default() -> Self {
        Self::new()
    }
}

impl Fork {
    pub fn new() -> Self {
        Self {
            host: Host::None,
            upstream: "".to_string(),
            owner: "".to_string(),
            repo: "".to_string(),
            repo_path: "".to_string(),
        }
    }
    pub fn from(host: Host, upstream: String, owner: String, repo: String) -> Self {
        Self {
            host,
            upstream,
            owner,
            repo,
            repo_path: String::new(),
        }
    }
    pub fn url(&self) -> String {
        format!("{}/{}/{}", self.host.url(), self.owner, self.repo)
    }

    pub fn clone_repo(&self) -> Result<String> {
        let path = format!(
            "{}/Developer/{}/{}/{}",
            home().display(),
            self.host,
            self.owner,
            self.repo
        );
        println!("Cloning {}/{} from {}...", self.owner, self.repo, self.host);
        Repository::clone(self.url().as_str(), &path).with_context(|| FAILED_TO_CLONE_REPO)?;
        Ok(path)
    }
    pub fn parse_url(url: &str, rx: Regex, upstream: String) -> Result<Self> {
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
        Ok(Self::from(Host::from(host.into()), upstream, owner, repo))
    }

    pub fn set_upstream(&self, path: String) -> Result<()> {
        println!("Setting {} how upstream...", self.upstream);
        if path.is_empty() {
            println!("It seems that you do not have cloned the repository locally");
        }
        let project = Repository::open(Path::new(&path)).expect(NO_PROJECT_FOUND);
        project
            .remote("upstream", &self.upstream)
            .with_context(|| FAILED_TO_SET_REMOTE)?;
        Ok(())
    }
}
