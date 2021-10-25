use anyhow::{Context, Result};
use git2::Repository;
use regex::bytes::Regex;
use std::path::Path;

use crate::home;
use crate::utils::constants::messages::*;
use crate::utils::host::Host;

pub struct Fork {
    pub host: Host,
    pub host_upstream: Host,
    pub owner_upstream: String,
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
            host_upstream: Host::None,
            owner_upstream: "".to_string(),
            owner: "".to_string(),
            repo: "".to_string(),
            repo_path: "".to_string(),
        }
    }
    pub fn from(
        host: Host,
        host_upstream: Host,
        owner_upstream: String,
        owner: String,
        repo: String,
    ) -> Self {
        Self {
            host,
            host_upstream,
            owner_upstream,
            owner,
            repo,
            repo_path: String::new(),
        }
    }
    pub fn url(&self) -> String {
        format!("{}/{}/{}", self.host.url(), self.owner, self.repo)
    }
    pub fn url_upstream(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host_upstream.url(),
            self.owner_upstream,
            self.repo
        )
    }

    pub fn clone_repo(&mut self) -> Result<()> {
        let path = format!(
            "{}/Developer/{}/{}/{}",
            home().display(),
            self.host,
            self.owner,
            self.repo
        );
        println!("Cloning {}/{} from {}...", self.owner, self.repo, self.host);
        Repository::clone(self.url().as_str(), &path).with_context(|| FAILED_TO_CLONE_REPO)?;
        self.repo_path = path;
        Ok(())
    }
    pub fn parse_url(url: &str, rx: Regex) -> Result<Self> {
        let captures = rx.captures(url.as_ref()).unwrap();
        let host = captures
            .get(4)
            .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
            .with_context(|| UNABLE_TO_MAP_URL)?;
        let upstream = captures
            .get(4)
            .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
            .with_context(|| UNABLE_TO_MAP_URL)?;
        let owner = captures
            .get(6)
            .map(|m| String::from_utf8(Vec::from(m.as_bytes())).unwrap())
            .with_context(|| UNABLE_TO_MAP_URL)?;
        let owner_upstream = captures
            .get(6)
            .map(|m| String::from_utf8(Vec::from(m.as_bytes())).unwrap())
            .with_context(|| UNABLE_TO_MAP_URL)?;
        let repo = captures
            .get(7)
            .map(|m| String::from_utf8(Vec::from(m.as_bytes())).unwrap())
            .with_context(|| UNABLE_TO_MAP_URL)?;
        Ok(Self::from(
            Host::from(host.into()),
            Host::from(upstream.into()),
            owner,
            owner_upstream,
            repo,
        ))
    }

    pub fn set_upstream(&self) -> Result<()> {
        println!("Setting {} how upstream...", self.host_upstream);
        if self.repo_path.is_empty() {
            println!("It seems that you do not have cloned the repository locally");
        }
        let project = Repository::open(Path::new(&self.repo_path)).expect(NO_PROJECT_FOUND);
        project
            .remote("upstream", &self.url_upstream())
            .with_context(|| FAILED_TO_SET_REMOTE)?;
        Ok(())
    }
}
