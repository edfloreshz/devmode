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
    /// Register an existing on-disk repo into devmode's registry.
    Track {
        /// Path to the repo directory.
        path: std::path::PathBuf,
        /// Tags to attach to this repo.
        #[arg(long)]
        tag: Vec<String>,
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
}
