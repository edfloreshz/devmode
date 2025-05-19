use crate::Error;
use clap::{Parser, Subcommand};
use colored::*;
use devmode::{services, CliError};

#[derive(Parser, Debug)]
#[clap(name = "Devmode")]
#[clap(about = "Devmode is a project management utility for developers.")]
#[clap(author, version, about, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Clones a repository in a specific folder structure.",
        name = "cl"
    )]
    Clone {
        #[arg(help = "Provide either a Git <url> or a Git <host> <owner> <repo>.")]
        url: String,
    },
}

impl Cli {
    pub fn run(&self) -> Result<(), Error> {
        match &self.commands {
            Commands::Clone { url } => match services::clone(&url) {
                Ok(_) => {
                    println!(
                        "{} {}",
                        "success:".green().bold(),
                        format!("Repository cloned to {}", url).green()
                    );
                    Ok(())
                }
                Err(services::Error::Clone(services::CloneError::PathExists(path))) => {
                    if overwrite() {
                        std::fs::remove_dir_all(&path)?;
                        println!(
                            "{} {}",
                            "info:".cyan().bold(),
                            format!("Removing existing repository at {}", path.display()).cyan()
                        );
                        println!(
                            "{} {}",
                            "info:".cyan().bold(),
                            format!("Cloning {}...", url).cyan()
                        );
                        services::clone(&url)?;
                        println!(
                            "{} {}",
                            "success:".green().bold(),
                            format!("Repository cloned to {}", path.display()).green()
                        );
                        Ok(())
                    } else {
                        Err(CliError::RepositoryExists.into())
                    }
                }
                Err(e) => Err(e.into()),
            },
        }
    }
}

fn overwrite() -> bool {
    println!(
        "{} Found existing repository, overwrite it? y/n",
        "prompt:".yellow().bold()
    );
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim(), "y" | "Y")
}
