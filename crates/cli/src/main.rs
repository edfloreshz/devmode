mod cli;
mod commands;
mod error;
mod prompt;
mod resolve;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Config { command } => commands::config::run(command),
        Command::Repo { command } => commands::repo::run(command),
        Command::Workspace { command } => commands::workspace::run(command),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
