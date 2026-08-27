mod cli;
mod commands;
mod error;
mod prompt;
mod resolve;

use clap::{CommandFactory, Parser};
use cli::{Cli, Command, RepoCommand};
use colored::Colorize;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Config { command } => commands::config::run(command),
        Command::Repo { command } => commands::repo::run(command),
        Command::Workspace { command } => commands::workspace::run(command),
        Command::Clone { url, path } => commands::repo::run(RepoCommand::Clone { url, path }),
        Command::Ls { tag, host, json } => {
            commands::repo::run(RepoCommand::List { tag, host, json })
        }
        Command::Find { query } => commands::repo::run(RepoCommand::Find { query }),
        Command::Completions { shell } => {
            let mut command = Cli::command();
            let name = command.get_name().to_string();
            clap_complete::generate(shell, &mut command, name, &mut std::io::stdout());
            Ok(())
        }
    };

    if let Err(err) = result {
        if dm_util::output::use_color_stderr() {
            eprintln!("{} {err}", "error:".red().bold());
        } else {
            eprintln!("error: {err}");
        }
        std::process::exit(1);
    }
}
