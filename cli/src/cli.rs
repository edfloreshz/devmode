use crate::Error;
use clap::{Parser, Subcommand};
use devmode_cli::CliError;

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
                    log::info!("Repository cloned to {}", url);
                    Ok(())
                }
                Err(services::Error::Clone(services::CloneError::PathExists(path))) => {
                    if overwrite() {
                        std::fs::remove_dir_all(&path)?;
                        log::info!("Removing existing repository at {}", path.display());
                        log::info!("Cloning {}...", url.to_string());
                        services::clone(&url)?;
                        log::info!("Repository cloned to {}", path.display());
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
    println!("Found existing repository, overwrite it? y/n");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim(), "y" | "Y")
}
