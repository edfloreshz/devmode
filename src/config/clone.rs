use std::path::PathBuf;
use crate::config::host::Host;
use crate::constants::messages::*;
use anyhow::{bail, Context, Result};
use git2_credentials::CredentialHandler;
use libdmd::home;
use regex::bytes::Regex;

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
    pub fn from(host: Host, owner: String, repos: Vec<String>) -> Self {
        CloneAction { host, owner, repos }
    }
    pub fn url(&self, index: usize) -> String {
        format!(
            "{}/{}/{}",
            self.host.url(),
            self.owner,
            self.repos.get(index).unwrap()
        )
    }
    pub fn run(&self) -> Result<()> {
        if let Host::None = self.host {
            bail!("You can't do this unless you set your configuration with `dmd config -a`\n\
                    In the meantime, you can clone by specifying <host> <owner> <repo>")
        } else if self.owner.is_empty() {
            bail!("Missing arguments: <owner> <repo>")
        } else if self.repos.is_empty() {
            bail!("Missing arguments: <repo>")
        } else {
            self.clone_repo()
        }
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
            {
                let mut cb = git2::RemoteCallbacks::new();
                let git_config = git2::Config::open_default()?;
                let mut ch = CredentialHandler::new(git_config);
                cb.credentials(move |url, username, allowed| ch.try_next_credential(url, username, allowed));

                let mut fo = git2::FetchOptions::new();
                fo.remote_callbacks(cb)
                    .download_tags(git2::AutotagOption::All)
                    .update_fetchhead(true);
                std::fs::create_dir_all(PathBuf::from(&path))?;
                git2::build::RepoBuilder::new()
                    .branch(match self.host {
                        Host::GitHub => "main",
                        Host::GitLab => "master",
                        Host::None => "master"
                    })
                    .fetch_options(fo)
                    .clone(self.url(ix).as_str(), &*PathBuf::from(&path)).with_context(|| FAILED_TO_CLONE_REPO)?;
            }
        }
        Ok(())
    }
    pub fn parse_url(url: &str, rx: Regex) -> Result<CloneAction> {
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
        Ok(CloneAction::from(
            Host::from(host.into()),
            owner,
            vec![repo],
        ))
    }
}
