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
    pub url: GitUrl,
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
            url: GitUrl::parse(url).unwrap(),
            workspace: None,
        }
    }

    pub fn clone_repo(&self) -> Result<(), Error> {
        let path = self.get_local_path()?;

        let clone = git2::build::RepoBuilder::new()
            .fetch_options(CloneAction::get_fetch_options()?)
            .clone(&self.url.to_string(), &path);

        if let Err(err) = clone {
            if let ErrorCode::NotFound = err.code() {
                remove_dir_all(&path)?
            }
            return Err(Error::Git(err));
        }

        git_pull::status_short(path.to_str().unwrap().to_string())?;
        OpenAction::make_dev_paths()?;
        Ok(())
    }

    pub fn get_local_path(&self) -> Result<PathBuf, Error> {
        if self.url.host.is_none() || self.url.owner.is_none() {
            return error::generic("Url is not in the correct format.");
        }
        let path = home()
            .join("Developer")
            .join(Host::from(self.url.host.as_ref().unwrap()).to_string())
            .join(self.url.owner.as_ref().unwrap())
            .join(self.workspace.as_ref().unwrap_or(&String::default()))
            .join(self.url.name.clone());

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
