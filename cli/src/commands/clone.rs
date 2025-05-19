// Command module for 'clone'
use git2_credentials::CredentialHandler;
use git_url_parse::GitUrl;

use super::{CloneError, Error};

pub fn run(url: &str) -> Result<(), Error> {
    let url = GitUrl::parse(url)?;
    let path = match (&url.host, &url.owner, &url.name) {
        (Some(host), Some(owner), name) if !owner.is_empty() => dirs::home_dir()
            .unwrap()
            .join("Developer")
            .join(host)
            .join(owner)
            .join(name),
        _ => return Err(CloneError::InvalidUrl.into()),
    };
    if path.exists() {
        return Err(CloneError::PathExists(path).into());
    }
    let mut cb = git2::RemoteCallbacks::new();
    let config = git2::Config::open_default()?;
    let mut credential_handler = CredentialHandler::new(config);
    cb.credentials(move |url, username, allowed| {
        credential_handler.try_next_credential(url, username, allowed)
    });
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(cb);
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);
    builder.clone(url.to_string().as_str(), &path)?;
    Ok(())
}
