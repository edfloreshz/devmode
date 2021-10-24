use anyhow::{Context, Result};
use git2::Repository;
use regex::bytes::Regex;
use std::path::Path;

use crate::home;
use crate::utils::constants::messages::*;
use crate::utils::host::Host;

pub struct Fork {
    pub host: Host,
    pub upstream: Host,
    pub owner: String,
    pub repo: String,
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
            upstream: Host::None,
            owner: "".to_string(),
            repo: "".to_string(),
        }
    }
    pub fn from(host: Host, upstream: Host, owner: String, repo: String) -> Self {
        Self {
            host,
            upstream,
            owner,
            repo,
        }
    }
    pub fn url(&self) -> String {
        format!("{}/{}/{}", self.host.url(), self.owner, self.repo)
    }

    pub fn clone_repo(&self) -> Result<()> {
        let path = format!(
            "{}/Developer/{}/{}/{}",
            home().display(),
            self.host,
            self.owner,
            self.repo
        );
        println!("Cloning {}/{} from {}...", self.owner, self.repo, self.host);
        Repository::clone(self.url().as_str(), &path).with_context(|| FAILED_TO_CLONE_REPO)?;
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
        let repo = captures
            .get(7)
            .map(|m| String::from_utf8(Vec::from(m.as_bytes())).unwrap())
            .with_context(|| UNABLE_TO_MAP_URL)?;
        Ok(Self::from(
            Host::from(host.into()),
            Host::from(upstream.into()),
            owner,
            repo,
        ))
    }

    pub fn set_upstream(&self, path_project: String) -> Result<()> {
        println!("Setting {} how upstream...", self.upstream);
        let project = Repository::open(Path::new(&path_project)).expect(NO_PROJECT_FOUND);
        project
            .remote("upstream", self.upstream.url())
            .with_context(|| FAILED_TO_SET_REMOTE)?;
        Ok(())
    }
}
