use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "dm",
    version,
    about = "Devmode: a project management utility for developers"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Manage devmode configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Manage tracked repos.
    Repo {
        #[command(subcommand)]
        command: RepoCommand,
    },
    /// Clone a repo from a URL and track it (shortcut for `dm repo clone`).
    Clone {
        url: String,
        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },
    /// List tracked repos (shortcut for `dm repo list`).
    Ls {
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Search tracked repos by name (shortcut for `dm repo find`).
    Find { query: String },
    /// Print shell completions for the given shell.
    Completions { shell: clap_complete::Shell },
    /// Manage workspaces — named, non-destructive groups of repos.
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Print the value of a config key.
    Get { key: String },
    /// Set the value of a config key.
    Set { key: String, value: String },
    /// Show the full effective configuration.
    Show {
        /// Output as JSON instead of TOML.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum RepoCommand {
    /// Clone a repo from a URL (HTTP(S) or SSH) and track it.
    Clone {
        /// The URL to clone.
        url: String,
        /// Exact destination directory (overrides the configured layout).
        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },
    /// Create a new local repo and track it.
    Create {
        /// Name for the new repo.
        name: String,
        /// Exact destination directory (overrides the default local placement).
        #[arg(long)]
        path: Option<std::path::PathBuf>,
        /// Don't run `git init` — just create the directory.
        #[arg(long)]
        no_git: bool,
    },
    /// Show details about a tracked repo.
    Show {
        /// Repo name or path.
        repo: String,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Untrack (and optionally delete) a repo.
    Remove {
        /// Repo name or path.
        repo: String,
        /// Also delete the repo directory from disk.
        #[arg(long)]
        delete: bool,
        /// Skip the delete confirmation prompt.
        #[arg(short, long)]
        force: bool,
    },
    /// Register an existing on-disk repo into devmode's registry.
    Track {
        /// Path to the repo directory.
        path: std::path::PathBuf,
        /// Tags to attach to this repo.
        #[arg(long)]
        tag: Vec<String>,
        /// Host this repo belongs to (e.g. github.com) — needed for `dm repo relayout` to manage it.
        #[arg(long)]
        host: Option<String>,
        /// Owner/org this repo belongs to — needed for `dm repo relayout` to manage it.
        #[arg(long)]
        owner: Option<String>,
    },
    /// List tracked repos.
    List {
        /// Only show repos with this tag.
        #[arg(long)]
        tag: Option<String>,
        /// Only show repos on this host (e.g. github.com).
        #[arg(long)]
        host: Option<String>,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Preview or apply moving tracked repos to match the current layout.
    Relayout {
        /// Actually move repos on disk and update the registry (default is a dry-run preview).
        #[arg(long)]
        apply: bool,
        /// Skip the confirmation prompt when applying.
        #[arg(short, long)]
        yes: bool,
    },
    /// Scan a directory tree for untracked repos and offer to track them.
    Scan {
        /// Directory to scan (defaults to repo.root).
        root: Option<std::path::PathBuf>,
        /// Track every repo found without asking.
        #[arg(short, long)]
        yes: bool,
    },
    /// Check tracked repos against disk: still exist? remote changed?
    #[command(alias = "doctor")]
    Sync {
        /// Apply fixes (untrack missing, update changed remotes) without asking.
        #[arg(short, long)]
        yes: bool,
    },
    /// Search tracked repos by name.
    Find {
        /// Text to match against repo names.
        query: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum WorkspaceCommand {
    /// Create a new workspace.
    Create {
        /// Slug identifying the workspace (used everywhere else as `<workspace>`).
        id: String,
        /// Display name (defaults to the id).
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        /// Command used to open member repos on `switch`, e.g. "code -n".
        #[arg(long)]
        editor: Option<String>,
    },
    /// Add repo(s) (by name or path) to a workspace.
    Add {
        workspace: String,
        repos: Vec<String>,
    },
    /// Remove repo(s) (by name or path) from a workspace.
    Remove {
        workspace: String,
        repos: Vec<String>,
    },
    /// List workspaces.
    List {
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show a workspace's details, members, and env vars.
    Show {
        workspace: String,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Get/set a workspace's name, description, or editor.
    Config {
        #[command(subcommand)]
        command: WorkspaceConfigCommand,
    },
    /// Get/set/list per-workspace environment variables, applied on `switch`.
    Env {
        #[command(subcommand)]
        command: WorkspaceEnvCommand,
    },
    /// Open the workspace's editor with all member repos, with its env vars applied.
    Switch {
        workspace: String,
        /// Print `cd <path>` for the first member instead of opening an editor
        /// (for `eval "$(dm workspace switch <id> --cd)"`).
        #[arg(long)]
        cd: bool,
    },
    /// Delete a workspace. Member repos themselves are not affected.
    Delete {
        workspace: String,
        /// Skip the confirmation prompt.
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum WorkspaceConfigCommand {
    /// Print the value of a workspace config key (name, description, editor).
    Get { workspace: String, key: String },
    /// Set a workspace config key (name, description, editor).
    Set {
        workspace: String,
        key: String,
        value: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum WorkspaceEnvCommand {
    /// Set an environment variable for a workspace.
    Set {
        workspace: String,
        key: String,
        value: String,
    },
    /// Unset an environment variable for a workspace.
    Unset { workspace: String, key: String },
    /// List a workspace's environment variables.
    List { workspace: String },
}
