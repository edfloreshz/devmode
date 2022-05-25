use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use git2_credentials::CredentialHandler;
use libset::routes::home;
use regex::bytes::Regex;

use crate::config::host::Host;
use crate::constants::messages::*;

pub struct CloneAction {
    pub host: Host,
    pub owner: String,
    pub repos: Vec<String>,
}

impl Default for CloneAction {
    fn default() -> Self {
        Self::new()
    }
}

impl CloneAction {
    pub fn new() -> Self {
        CloneAction {
            host: Host::None,
            owner: String::new(),
            repos: Vec::new(),
        }
    }
    pub fn from(host: Host, owner: &str, repos: Vec<String>) -> Self {
        let owner = owner.to_string();
        let repos = repos.iter().map(|r| r.to_string()).collect();
        CloneAction { host, owner, repos }
    }
    pub fn url(&self, index: usize) -> Result<String> {
        let url = format!(
            "{}/{}/{}",
            self.host.url(),
            self.owner,
            self.repos
                .get(index)
                .with_context(|| "Failed to get url from index.")?
        );
        Ok(url)
    }
    pub fn run(&self) -> Result<()> {
        if let Host::None = self.host {
            bail!(
                "You can't do this unless you set your configuration with ` dm config -a`\n\
                    In the meantime, you can clone by specifying <host> <owner> <repo>"
            )
        } else if self.owner.is_empty() {
            bail!("Missing arguments: <owner> <repo>")
        } else if self.repos.is_empty() {
            bail!("Missing arguments: <repo>")
        } else {
            self.clone_repo()
        }
    }
    pub fn clone_repo(&self) -> Result<()> {
        let mut error = anyhow!("");
        for (ix, repo) in self.repos.iter().enumerate() {
            let path = format!(
                "{}/Developer/{}/{}/{}",
                home().display(),
                self.host,
                self.owner,
                repo
            );
            println!("Cloning {}/{} from {}...", self.owner, repo, self.host);
            let mut cb = git2::RemoteCallbacks::new();
            let git_config = git2::Config::open_default()?;
            let mut ch = CredentialHandler::new(git_config);
            cb.credentials(move |url, username, allowed| {
                ch.try_next_credential(url, username, allowed)
            });

            let mut fo = git2::FetchOptions::new();
            fo.remote_callbacks(cb)
                .download_tags(git2::AutotagOption::All)
                .update_fetchhead(true);
            std::fs::create_dir_all(PathBuf::from(&path))?;

            if let Err(e) = git2::build::RepoBuilder::new()
                .fetch_options(fo)
                .clone(self.url(ix)?.as_str(), &*PathBuf::from(&path))
                .with_context(|| FAILED_TO_CLONE_REPO)
            {
                error = e;
            } else if self.repos.len() == ix + 1 {
                return Ok(());
            }
        }
        Err(error)
    }
    pub fn parse_url(url: &str, rx: Regex) -> Result<CloneAction> {
        let captures = rx
            .captures(url.as_ref())
            .with_context(|| "Failed to get captures from url.")?;
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
        Ok(CloneAction::from(Host::from(host), &owner, vec![repo]))
    }
}
