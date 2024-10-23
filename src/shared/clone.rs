use std::fs::remove_dir_all;
use std::path::PathBuf;

use derive_setters::*;
use git2::ErrorCode;
use git2_credentials::CredentialHandler;
use git_url_parse::GitUrl;
use libset::routes::home;

use crate::action::Action;
use crate::host::Host;
use crate::project::OpenAction;
use crate::{error, git_pull, Error};

#[derive(Debug, Default, Clone, Setters)]
#[setters(prefix = "set_")]
#[setters(borrow_self)]
#[setters(strip_option)]
pub struct CloneAction {
    #[setters(skip)]
    pub url: Option<GitUrl>,
    pub workspace: Option<String>,
}

impl Action for CloneAction {
    fn run(&mut self) -> Result<(), Error> {
        self.clone_repo()
    }
}

impl CloneAction {
    pub fn new(url: &str) -> Self {
        Self {
            url: GitUrl::parse(url).ok(),
            workspace: None,
        }
    }

    pub fn clone_repo(&self) -> Result<(), Error> {
        let Some(url) = &self.url else {
            return error::error("Url is not in the correct format.");
        };
        let path = self.get_local_path()?;
        let clone = git2::build::RepoBuilder::new()
            .fetch_options(CloneAction::get_fetch_options()?)
            .clone(&url.to_string(), &path);

        if let Err(err) = clone {
            match err.code() {
                ErrorCode::GenericError => {
                    if let Some(parent) = path.parent() {
                        let children: Vec<_> = std::fs::read_dir(parent)?.collect();
                        if children.is_empty() {
                            remove_dir_all(&parent)?;
                        }
                    }
                }
                _ => return Err(Error::Git(err)),
            }
        }

        git_pull::status_short(path.to_str().unwrap().to_string())?;
        OpenAction::make_dev_paths()?;
        Ok(())
    }

    pub fn get_local_path(&self) -> Result<PathBuf, Error> {
        let Some(url) = &self.url else {
            return error::error("Url is not in the correct format.");
        };
        if url.host.is_none() || url.owner.is_none() {
            return error::error("Url is not in the correct format.");
        }
        let path = home()
            .join("Developer")
            .join(Host::from(url.host.as_ref().unwrap()).to_string())
            .join(url.owner.as_ref().unwrap())
            .join(self.workspace.as_ref().unwrap_or(&String::default()))
            .join(url.name.clone());

        Ok(path)
    }

    pub fn get_fetch_options<'a>() -> Result<git2::FetchOptions<'a>, Error> {
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
