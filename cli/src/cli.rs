use clap::{Parser, Subcommand};
use dm_core::{
    error::{CloneError, WorkspaceError},
    workspace::find_repos,
};
use std::io::{self, Write};

use crate::{
    error::{CliError, Error},
    helpers, log,
};

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
        #[arg(
            long,
            help = "Assign the cloned repository to a workspace after cloning."
        )]
        workspace: Option<String>,
    },
    #[command(about = "Workspace management", name = "ws")]
    Workspace {
        #[command(subcommand)]
        subcommand: WorkspaceSubcommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceSubcommand {
    #[command(
        about = "Create a workspace at the same level as cloned repositories.",
        name = "create"
    )]
    Create {
        #[arg(help = "Name of the workspace to create.")]
        name: String,
    },
    #[command(
        about = "Assign a cloned repository to a workspace (move it into the workspace folder).",
        name = "assign"
    )]
    Assign {
        #[arg(help = "Path to the repository to assign.")]
        repo_path: String,
        #[arg(help = "Name of the workspace.")]
        workspace: String,
    },
    #[command(
        about = "Remove a repository from a workspace (move it out of the workspace folder).",
        name = "remove"
    )]
    Remove {
        #[arg(help = "Path to the repository inside the workspace.")]
        repo_in_ws: String,
        #[arg(help = "Name of the workspace.")]
        workspace: String,
    },
}

impl Cli {
    pub fn run(&self) -> Result<(), Error> {
        match &self.commands {
            Commands::Clone { url, workspace } => self.clone(url, workspace.as_deref()),
            Commands::Workspace { subcommand } => match subcommand {
                WorkspaceSubcommand::Create { name } => self.create_workspace(name),
                WorkspaceSubcommand::Assign {
                    repo_path,
                    workspace,
                } => self.assign_workspace(repo_path, workspace),
                WorkspaceSubcommand::Remove {
                    repo_in_ws,
                    workspace,
                } => self.remove_workspace(repo_in_ws, workspace),
            },
        }
    }

    fn clone(&self, url: &str, workspace: Option<&str>) -> Result<(), Error> {
        let clone_result = dm_core::clone::run(url);
        match clone_result {
            Ok(cloned_path) => {
                let repository = cloned_path
                    .file_name()
                    .ok_or_else(|| WorkspaceError::InvalidRepoPath)?;
                if let Some(ws_name) = workspace {
                    match dm_core::workspace::assign_repo_to_workspace(
                        &cloned_path,
                        &repository.to_string_lossy(),
                        ws_name,
                    ) {
                        Ok(()) => log::success(&format!(
                            "Repository cloned and assigned to workspace {}",
                            ws_name
                        )),
                        Err(e) => log::warning(&format!(
                            "Repository cloned, but failed to assign to workspace: {}",
                            e
                        )),
                    }
                } else {
                    log::success(&format!("Repository cloned to {}", cloned_path.display()));
                }
                Ok(())
            }
            Err(CloneError::PathExists(path)) => {
                if helpers::overwrite() {
                    std::fs::remove_dir_all(&path)?;
                    log::warning(&format!(
                        "Removing existing repository at {}",
                        path.display()
                    ));
                    log::info(&format!("Cloning {}...", url));
                    self.clone(url, workspace)
                } else {
                    Err(CliError::RepositoryExists.into())
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn create_workspace(&self, name: &str) -> Result<(), Error> {
        match dm_core::workspace::create_workspace(name) {
            Ok(ws_path) => {
                log::success(&format!("Created workspace at {}", ws_path.display()));
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn assign_workspace(&self, repository: &str, workspace: &str) -> Result<(), Error> {
        let mut matches = vec![];
        find_repos(None, repository, &mut matches);
        let matches: Vec<_> = matches
            .into_iter()
            .filter(|p| {
                let name = p.file_name().map(|n| n.to_string_lossy().to_lowercase());
                let is_workspace_folder =
                    name.as_deref().map_or(false, |n| n.contains("workspace"));
                let is_inside_workspace = p
                    .components()
                    .any(|c| c.as_os_str().to_string_lossy().to_lowercase() == "workspaces");
                !is_workspace_folder && !is_inside_workspace
            })
            .collect();
        if matches.is_empty() {
            log::error(&format!("No repositories found matching '{}'.", repository));
            return Ok(());
        }
        let selected_repo = if matches.len() == 1 {
            &matches[0]
        } else {
            log::info("Found repositories:");
            for (i, path) in matches.iter().enumerate() {
                log::info(&format!("  [{}] {}", i + 1, path.display()));
            }
            log::info(&format!(
                "Select a repository to assign (1-{}): ",
                matches.len()
            ));
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let idx: usize = input.trim().parse().unwrap_or(0);
            if !(1..=matches.len()).contains(&idx) {
                log::error("Invalid selection.");
                return Ok(());
            }
            &matches[idx - 1]
        };
        if let Some(repo_name) = selected_repo.file_name().and_then(|n| n.to_str()) {
            let _ = dm_core::workspace::update_workspace_metadata_on_assign(
                workspace,
                repo_name,
                selected_repo,
            );
        }
        match dm_core::workspace::assign_repo_to_workspace(selected_repo, repository, workspace) {
            Ok(()) => {
                log::success(&format!("Assigned repository to workspace {}", workspace));
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn remove_workspace(&self, repository: &str, workspace: &str) -> Result<(), Error> {
        let original_path =
            dm_core::workspace::get_original_path_from_metadata(workspace, repository)
                .ok_or_else(|| Error::from(CliError::OriginalRepositoryPathNotFound))?;
        match dm_core::workspace::remove_repo_from_workspace(&original_path, repository, workspace)
        {
            Ok(_) => {
                log::success(&format!(
                    "Removed repository from workspace {} and restored it to {}",
                    workspace,
                    original_path.display()
                ));
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}
