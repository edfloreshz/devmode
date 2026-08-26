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
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Print the value of a config key.
    Get { key: String },
    /// Set the value of a config key.
    Set { key: String, value: String },
}
