use anyhow::{Context, Result};
use git2::Repository;
use regex::bytes::Regex;

use crate::home;
use crate::utils::constants::messages::*;
use crate::utils::host::Host;

pub struct Clone<'a> {
    pub host: Option<Host<'a>>,
    pub owner: Option<String>,
    pub repo: Option<String>,
}

impl<'a> Clone<'a> {
    pub fn new(host: Option<Host<'a>>, owner: Option<String>, repo: Option<String>) -> Self {
        Clone { host, owner, repo }
    }
    pub fn url(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host.unwrap().url(),
            self.owner.as_ref().unwrap(),
            self.repo.as_ref().unwrap()
        )
    }
    pub fn clone_repo(&self) -> Result<()> {
        let path = format!(
            "{}/Developer/{}/{}/{}",
            home().display(),
            self.host.unwrap(),
            self.owner.as_ref().unwrap(),
            self.repo.as_ref().unwrap()
        );
        println!(
            "Cloning {}/{} from {}...",
            self.owner.as_ref().unwrap(),
            self.repo.as_ref().unwrap(),
            self.host.unwrap()
        );
        Repository::clone(self.url().as_str(), &path)
            .with_context(|| FAILED_TO_CLONE_REPO)?;
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
        Ok(Clone::new(
            Host::from(host.into()),
            Option::from(owner),
            Option::from(repo),
        ))
    }
}
