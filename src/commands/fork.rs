use std::path::Path;

use anyhow::{bail, Context, Result};
use git2::Repository;
use libset::routes::home;
use regex::bytes::Regex;

use crate::commands::host::Host;
use crate::commands::project::OpenAction;
use crate::constants::messages::*;

pub struct ForkAction {
    pub host: Host,
    pub upstream: String,
    pub owner: String,
    pub repo: String,
    pub repo_path: String,
}

impl Default for ForkAction {
    fn default() -> Self {
        Self::new()
    }
}

impl ForkAction {
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
    pub fn run(&self) -> Result<()> {
        if let Host::None = self.host {
            bail!(
                "You can't do this unless you set your configuration with ` dm config -a`\n\
                    In the meantime, you can clone by specifying <host> <owner> <repo>"
            )
        } else if self.owner.is_empty() {
            bail!("Missing arguments: <owner> <repo>")
        } else if self.repo.is_empty() {
            bail!("Missing arguments: <repo>")
        } else if self.upstream.is_empty() {
            bail!(
                "Missing arguments: <upstream>. \
            For example ... -u https://github.com/user/upstream"
            )
        } else {
            match self.clone_repo() {
                Ok(path) => {
                    OpenAction::make_dev_paths()?;
                    self.set_upstream(path)
                }
                Err(e) => Err(e),
            }
        }
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
        let captures = rx
            .captures(url.as_ref())
            .with_context(|| "Failed to get url captures.")?;
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
        Ok(Self::from(Host::from(host), upstream, owner, repo))
    }

    pub fn set_upstream(&self, path: String) -> Result<()> {
        println!("Setting {} as upstream...", self.upstream);
        if path.is_empty() {
            println!("Seems that you haven't cloned the repository locally.");
        }
        let project = Repository::open(Path::new(&path)).expect(NO_PROJECT_FOUND);
        project
            .remote("upstream", &self.upstream)
            .with_context(|| FAILED_TO_SET_REMOTE)?;
        Ok(())
    }
}
