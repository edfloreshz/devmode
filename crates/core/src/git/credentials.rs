use std::error::Error as StdError;

use git2::{Config, FetchOptions, RemoteCallbacks};
use git2_credentials::{CredentialHandler, CredentialUI};

/// Never prompts on stdin. devmode is meant to work non-interactively (it's
/// invoked from scripts, and blocking on a surprise username/password
/// prompt for e.g. a mistyped or private URL is bad UX); credentials should
/// come from an SSH agent, an SSH key file, or the system's git credential
/// helper, all of which `CredentialHandler` tries before falling back here.
struct NonInteractiveUi;

impl CredentialUI for NonInteractiveUi {
    fn ask_user_password(&self, _username: &str) -> Result<(String, String), Box<dyn StdError>> {
        Err("no username/password available (devmode doesn't prompt, \
             configure a git credential helper, or use an SSH URL with a key in your ssh-agent)"
            .into())
    }

    fn ask_ssh_passphrase(&self, _passphrase_prompt: &str) -> Result<String, Box<dyn StdError>> {
        Err("no SSH key passphrase available (devmode doesn't prompt, add the key to ssh-agent instead)".into())
    }
}

/// System-level gitconfig locations that vendor-specific git installs use
/// but libgit2's own default system-config search doesn't know about, most
/// notably Xcode's bundled git on macOS, which ships its system config
/// (setting `credential.helper = osxkeychain`) inside the app bundle rather
/// than a path libgit2 checks by default. Merging these in (still purely
/// via git2's own config file APIs, no shelling out to `git`) means devmode
/// picks up the same credential helper real `git` would use.
const VENDOR_SYSTEM_CONFIGS: &[&str] = &[
    "/Applications/Xcode.app/Contents/Developer/usr/share/git-core/gitconfig",
    "/Library/Developer/CommandLineTools/usr/share/git-core/gitconfig",
    "/opt/homebrew/etc/gitconfig",
    "/usr/local/etc/gitconfig",
];

fn open_git_config() -> Config {
    let mut cfg = Config::open_default().unwrap_or_else(|_| {
        Config::new().expect("git2 should always be able to build an empty in-memory config")
    });
    for path in VENDOR_SYSTEM_CONFIGS {
        let path = std::path::Path::new(path);
        if path.is_file() {
            let _ = cfg.add_file(path, git2::ConfigLevel::System, false);
        }
    }
    cfg
}

/// Fetch options wired to try SSH agent/key auth and HTTPS credential-helper
/// auth as needed, based on what the remote actually asks for, covers both
/// `https://` and `git@host:` remote URLs without devmode having to guess
/// the transport up front.
pub(super) fn fetch_options() -> FetchOptions<'static> {
    let git_config = open_git_config();
    let mut handler = CredentialHandler::new_with_ui(git_config, Box::new(NonInteractiveUi));
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username, allowed| {
        handler.try_next_credential(url, username, allowed)
    });
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    fetch_options
}
