use std::fs::remove_dir_all;
use std::path::PathBuf;
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

pub struct CloneAction {
    pub host: Host,
    pub owner: String,
    pub repos: Vec<String>,
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
            host: Host::None,
            owner: String::new(),
            repos: Vec::new(),
            url: None,
            workspace: None,
        }
    }
    pub fn from(host: Host, owner: &str, repos: Vec<String>, workspace: Option<String>) -> Self {
        let owner = owner.to_string();
        let repos = repos.iter().map(|r| r.to_string()).collect();
        CloneAction {
            host,
            owner,
            repos,
            url: None,
            workspace,
        }
    }
    pub fn from_url(url: &str, workspace: Option<String>) -> Result<Self> {
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
            let action = CloneAction {
                host: Host::from(host),
                owner: owner.to_string(),
                repos: vec![repo.to_string()],
                url: Some(url.to_string()),
                workspace,
            };
            Ok(action)
        } else {
            Err(anyhow!("Failed to parse URL."))
        }
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
        if self.url.is_some() {
            self.clone_repo_url()
        } else if let Host::None = self.host {
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
        for (index, repo) in self.repos.iter().enumerate() {
            let path = if self.workspace.is_some() {
                home()
                    .join("Developer")
                    .join(self.host.to_string())
                    .join(self.owner.clone())
                    .join(self.workspace.as_ref().unwrap())
                    .join(repo)
            } else {
                home()
                    .join("Developer")
                    .join(self.host.to_string())
                    .join(self.owner.clone())
                    .join(repo)
            };

            println!("Cloning {}/{} from {}...", self.owner, repo, self.host);

            std::fs::create_dir_all(PathBuf::from(&path))?;

            if let Err(err) = git2::build::RepoBuilder::new()
                .fetch_options(CloneAction::get_fetch_options()?)
                .clone(self.url(index)?.as_str(), &path)
            {
                match err.code() {
                    ErrorCode::NotFound => remove_dir_all(path)?,
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
                                remove_dir_all(&path)?;
                                println!("Retrying clone...");
                                git2::build::RepoBuilder::new()
                                    .fetch_options(CloneAction::get_fetch_options()?)
                                    .clone(self.url(index)?.as_str(), &path)?;
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

    pub fn clone_repo_url(&self) -> Result<()> {
        let path = if self.workspace.is_some() {
            home()
                .join("Developer")
                .join(self.host.to_string())
                .join(self.owner.clone())
                .join(self.workspace.as_ref().unwrap())
                .join(self.repos[0].clone())
        } else {
            home()
                .join("Developer")
                .join(self.host.to_string())
                .join(self.owner.clone())
                .join(self.repos[0].clone())
        };
        println!(
            "Cloning {}/{} from {}...",
            self.owner, self.repos[0], self.host
        );

        std::fs::create_dir_all(PathBuf::from(&path))?;

        if let Err(err) = git2::build::RepoBuilder::new()
            .fetch_options(CloneAction::get_fetch_options()?)
            .clone(self.url.as_ref().unwrap().as_str(), &path)
        {
            match err.code() {
                ErrorCode::NotFound => remove_dir_all(path)?,
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
                            remove_dir_all(&path)?;
                            println!("Retrying clone...");
                            git2::build::RepoBuilder::new()
                                .fetch_options(CloneAction::get_fetch_options()?)
                                .clone(self.url.as_ref().unwrap().as_str(), &path)?;
                        }
                    }
                }
                _ => return Err(err.into()),
            }
        }
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
