use std::fs::remove_dir_all;
use std::str::FromStr;

use anyhow::{anyhow, bail, Context, Ok, Result};
use colored::Colorize;
use git2::ErrorCode;
use git2_credentials::CredentialHandler;
use libset::routes::home;
use regex::bytes::Regex;

use crate::constants::messages::*;
use crate::constants::patterns::{ORG_GIT_URL, REGULAR_GIT_URL};
use crate::utils::host::Host;
use crate::utils::project::OpenAction;

#[derive(Clone)]
pub struct CloneAction {
    pub host: Option<Host>,
    pub owner: Option<String>,
    pub repos: Option<Vec<String>>,
    pub url: Option<String>,
    pub workspace: Option<String>,
}

impl Default for CloneAction {
    fn default() -> Self {
        Self::new()
    }
}

impl CloneAction {
    pub fn new() -> Self {
        CloneAction {
            host: None,
            owner: None,
            repos: Some(vec![]),
            url: None,
            workspace: None,
        }
    }
    pub fn set_host(mut self, host: Option<Host>) -> Self {
        self.host = host;
        self
    }
    pub fn set_owner(mut self, owner: Option<&String>) -> Self {
        self.owner = owner.cloned();
        self
    }
    pub fn set_repos(mut self, repos: Option<Vec<String>>) -> Self {
        self.repos = repos;
        self
    }
    pub fn set_url(mut self, url: Option<&String>) -> Result<Self> {
        self.url = url.cloned();
        Ok(self)
    }
    pub fn set_workspace(mut self, workspace: Option<String>) -> Self {
        self.workspace = workspace;
        self
    }
    pub fn url_from_index(&self, index: usize) -> Result<String> {
        let url = format!(
            "{}/{}/{}",
            self.host.clone().unwrap().url(),
            self.owner.clone().unwrap(),
            self.repos
                .clone()
                .unwrap()
                .get(index)
                .with_context(|| "Failed to get url from index.")?
        );
        Ok(url)
    }
    pub fn run(&mut self) -> Result<()> {
        if self.url.is_some() {
            self.parse_url()?;
            self.clone_repo()
        } else if let Host::None = self.host.clone().unwrap() {
            bail!(
                "You can't do this unless you set your configuration with ` dm config -a`\n\
                    In the meantime, you can clone by specifying <host> <owner> <repo>"
            )
        } else if self.owner.is_none() {
            bail!("Missing arguments: <owner> <repo>")
        } else if self.repos.is_none() {
            bail!("Missing arguments: <repo>")
        } else {
            self.clone_repo()
        }
    }
    pub fn parse_url(&mut self) -> Result<()> {
        let url = self.url.clone().unwrap();
        let regular = Regex::from_str(REGULAR_GIT_URL).with_context(|| "Failed to parse regex.")?;
        let organization =
            Regex::from_str(ORG_GIT_URL).with_context(|| "Failed to parse regex.")?;
        let captures = if regular.is_match(url.as_ref()) {
            Some(
                regular
                    .captures(url.as_ref())
                    .with_context(|| "Failed to parse URL.")?,
            )
        } else if organization.is_match(url.as_ref()) {
            Some(
                organization
                    .captures(url.as_ref())
                    .with_context(|| "Failed to parse URL.")?,
            )
        } else {
            None
        };
        if let Some(captures) = captures {
            let host = captures
                .name("host")
                .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
                .with_context(|| UNABLE_TO_MAP_URL)?;
            let owner = captures
                .name("owner")
                .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
                .with_context(|| UNABLE_TO_MAP_URL)?;
            let repo = captures
                .name("repo")
                .map(|m| std::str::from_utf8(m.as_bytes()).unwrap())
                .with_context(|| UNABLE_TO_MAP_URL)?;
            self.host = Some(Host::from(host));
            self.owner = Some(owner.to_string());
            self.repos = Some(vec![repo.to_string()]);
            Ok(())
        } else {
            Err(anyhow!("Failed to parse URL."))
        }
    }

    pub fn clone_repo(&self) -> Result<()> {
        let host = self.host.clone().unwrap();
        let owner = self.owner.clone().unwrap();
        let repos = self.repos.clone().unwrap();
        let path = home()
            .join("Developer")
            .join(host.to_string())
            .join(owner.clone());
        let path = if self.workspace.is_some() {
            path.join(self.workspace.as_ref().unwrap())
        } else {
            path
        };

        for (index, repo) in repos.iter().enumerate() {
            let save_path = if self.url.is_some() {
                path.join(repos[0].clone())
            } else {
                path.join(repo)
            };

            println!("Cloning {}/{} from {}...", owner, repo, host);

            std::fs::create_dir_all(&save_path)?;

            if let Err(err) = git2::build::RepoBuilder::new()
                .fetch_options(CloneAction::get_fetch_options()?)
                .clone(self.url_from_index(index)?.as_str(), &save_path)
            {
                match err.code() {
                    ErrorCode::NotFound => remove_dir_all(&save_path)?,
                    ErrorCode::Exists => {
                        println!(
                            "{}: {} exists and is not an empty directory",
                            Colorize::red("Error"),
                            path.display()
                        );
                        let question = requestty::Question::confirm("overwrite")
                            .message("Do you want to overwrite the existing repository?")
                            .build();
                        let answer = requestty::prompt_one(question)?;
                        if let requestty::Answer::Bool(overwrite) = answer {
                            if overwrite {
                                remove_dir_all(&save_path)?;
                                println!("Retrying clone...");
                                git2::build::RepoBuilder::new()
                                    .fetch_options(CloneAction::get_fetch_options()?)
                                    .clone(self.url_from_index(index)?.as_str(), &save_path)?;
                            }
                        }
                    }
                    _ => return Err(err.into()),
                }
            }
        }
        OpenAction::make_dev_paths()?;
        Ok(())
    }
    fn get_fetch_options<'a>() -> Result<git2::FetchOptions<'a>> {
        let mut callbacks = git2::RemoteCallbacks::new();
        let git_config = git2::Config::open_default()?;
        let mut credential_handler = CredentialHandler::new(git_config);
        callbacks.credentials(move |url, username, allowed| {
            credential_handler.try_next_credential(url, username, allowed)
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options
            .remote_callbacks(callbacks)
            .download_tags(git2::AutotagOption::All)
            .update_fetchhead(true);
        Ok(fetch_options)
    }
}
