use git2::{Config, FetchOptions, RemoteCallbacks};
use git2_credentials::CredentialHandler;

/// Fetch options wired to try SSH agent/key auth and HTTPS credential-helper
/// auth as needed, based on what the remote actually asks for — covers both
/// `https://` and `git@host:` remote URLs without devmode having to guess
/// the transport up front.
pub(super) fn fetch_options() -> FetchOptions<'static> {
    let git_config = Config::open_default().unwrap_or_else(|_| {
        Config::new().expect("git2 should always be able to build an empty in-memory config")
    });
    let mut handler = CredentialHandler::new(git_config);
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username, allowed| {
        handler.try_next_credential(url, username, allowed)
    });
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    fetch_options
}
