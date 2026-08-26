use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "dm", version, about = "Devmode: a project management utility for developers")]
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
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Print the value of a config key.
    Get { key: String },
    /// Set the value of a config key.
    Set { key: String, value: String },
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
}
