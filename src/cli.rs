use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use libset::routes::home;
use regex::bytes::Regex;
use std::fs;

use crate::constants::messages::APP_OPTIONS_NOT_FOUND;

use crate::utils::fork::ForkAction;
use crate::utils::host::Host;
use crate::utils::input::{clone_setup, fork_setup, config_all, config_editor, config_owner, config_host};
use crate::utils::project::OpenAction;
use crate::utils::settings::Settings;
use crate::{constants::patterns::GIT_URL, utils::clone::CloneAction};

#[derive(Parser, Debug)]
#[clap(name = "(Dev)mode", version = "0.2.9")]
#[clap(author = "Eduardo F. <edfloreshz@gmail.com>")]
#[clap(about = "Dev(mode) is a project management utility for developers.")]
#[clap(arg_required_else_help = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(
        about = "Clones a repository in a specific folder structure.",
        alias = "cl"
    )]
    Clone {
        #[clap(help = "Provide either a Git <url> or a Git <host> <owner> <repo>.")]
        #[clap(min_values = 1)]
        #[clap(max_values = 3)]
        args: Vec<String>,
        #[clap(
            help = "Select a workspace to store the repo in.",
            short = 'w',
            long = "workspace",
            takes_value = true
        )]
        workspace: Option<String>,
    },
    #[clap(
        about = "Opens a project on your selected text editor.",
        alias = "o",
        arg_required_else_help = true
    )]
    Open {
        #[clap(help = "Provide a project name")]
        #[clap(takes_value = true, required = true)]
        project: String,
    },
    #[clap(
        about = "Clones a repo and sets the upstream to your fork.",
        alias = "fk"
    )]
    Fork {
        #[clap(
            help = "Provide either a Git <url> or a Git <host> <owner> <repo>.",
            min_values = 1
        )]
        args: Vec<String>,
        #[clap(
            help = "Set the upstream to your fork <url>",
            short = 'u',
            long = "upstream"
        )]
        #[clap(takes_value = true, required = true)]
        upstream: String,
    },
    #[clap(
        about = "Write changes to your configuration.",
        alias = "cf",
        arg_required_else_help = true
    )]
    Config {
        #[clap(help = "Map your project paths.", short = 'm', long = "map")]
        map: bool,
        #[clap(help = "Show the current configuration.", short = 's', long = "show")]
        show: bool,
        #[clap(help = "Configure everything.", short = 'a', long = "all")]
        all: bool,
        #[clap(
            help = "Sets the favorite editor to open projects.",
            short = 'e',
            long = "editor"
        )]
        editor: bool,
        #[clap(
            help = "Sets the favorite owner to projects.",
            short = 'o',
            long = "owner"
        )]
        owner: bool,
        #[clap(help = "Sets the favorite host to clone projects.", long = "host")]
        host: bool,
    },
    #[clap(
        about = "Create workspaces to store your projects.",
        alias = "ws",
        arg_required_else_help = true
    )]
    Workspace {
        #[clap(help = "Name for the workspace.")]
        name: Option<String>,
        #[clap(help = "Delete a workspace", short = 'd', long = "delete")]
        delete: bool,
        #[clap(
            help = "Rename a workspace",
            short = 'r',
            long = "rename",
            takes_value = true
        )]
        rename: Option<String>,
        #[clap(help = "List all workspaces", short = 'l', long = "list")]
        list: bool,
    },
}

impl Cli {
    pub fn run(&self) -> anyhow::Result<()> {
        let rx = Regex::new(GIT_URL).with_context(|| "Unable to parse Regex.")?;
        match &self.commands {
            Commands::Clone { args, workspace } => Cli::clone(args, workspace.to_owned()),
            Commands::Open { project } => Cli::open(project),
            Commands::Fork { args, upstream } => Cli::fork(args, upstream, rx),
            Commands::Config {
                map,
                show,
                all,
                editor,
                owner,
                host,
            } => Cli::config(
                *map,
                *show,
                *all,
                *editor,
                *owner,
                *host,
                !map && !show && !*all && !*editor && !*owner && !*host,
            ),
            Commands::Workspace {
                name,
                delete,
                rename,
                list,
            } => Cli::workspace(name.to_owned(), *delete, rename.clone(), *list),
        }
    }
    fn clone(args: &[String], workspace: Option<String>) -> Result<()> {
        let clone = if args.is_empty() {
            clone_setup()?
        } else if args.len() == 1 && args.get(0).unwrap().contains("http") {
            CloneAction::from_url(args.get(0).unwrap(), workspace)?
        } else if args.len() == 3 {
            let host = Host::from(args.get(0).unwrap());
            let owner = args.get(1).unwrap();
            let repo = args.get(2).unwrap();
            CloneAction::from(host, owner, vec![repo.to_string()], workspace)
        } else {
            let options = Settings::current()
                .with_context(|| APP_OPTIONS_NOT_FOUND)?;
            CloneAction::from(
                Host::from(&options.host),
                &options.owner,
                args.to_vec(),
                workspace,
            )
        };
        clone.run()
    }
    fn open(project: &str) -> Result<()> {
        OpenAction::new(project).open()
    }
    fn fork(args: &[String], upstream: &str, rx: Regex) -> Result<()> {
        let action = if args.is_empty() {
            fork_setup()?
        } else if rx.is_match(args.get(0).unwrap().as_bytes()) {
            ForkAction::parse_url(args.get(0).unwrap(), rx, upstream.to_string())?
        } else if args.len() == 1 {
            let options = Settings::current()
                .with_context(|| APP_OPTIONS_NOT_FOUND)?;
            let host = Host::from(&options.host);
            let repo = args.get(0).map(|a| a.to_string());
            ForkAction::from(
                host,
                upstream.to_string(),
                options.owner,
                repo.with_context(|| "Failed to get repo name.")?,
            )
        } else {
            let host = Host::from(args.get(0).unwrap());
            let owner = args.get(1).map(|a| a.to_string());
            let repo = args.get(2).map(|a| a.to_string());
            ForkAction::from(
                host,
                upstream.to_string(),
                owner.with_context(|| "Failed to get owner name.")?,
                repo.with_context(|| "Failed to get repo name")?,
            )
        };
        action.run()
    }
    fn config(
        map: bool,
        show: bool,
        all: bool,
        editor: bool,
        owner: bool,
        host: bool,
        none: bool,
    ) -> Result<()> {
        if all || none {
            if get_settings().is_err() {
                println!("First time setup! 🥳\n");
                Settings::init()?;
            }
            let settings = config_all()?;
            settings.write()?;
        }
        if map {
            OpenAction::make_dev_paths()?
        }
        if editor {
            let settings = config_editor().with_context(|| "Failed to set editor.")?;
            settings.write()?
        }
        if owner {
            let settings = config_owner().with_context(|| "Failed to set owner.")?;
            settings.write()?
        }
        if host {
            let settings = config_host().with_context(|| "Failed to set host.")?;
            settings.write()?
        }
        if show {
            let settings = get_settings()?;
            settings.show();
        }
        Ok(())
    }
    fn workspace(
        name: Option<String>,
        delete: bool,
        rename: Option<String>,
        list: bool,
    ) -> Result<()> {
        let mut settings = Settings::current()
            .with_context(|| "Failed to get configuration")?;
        if let Some(name) = name {
            if settings.workspaces.names.contains(&name) {
                let index = settings
                    .workspaces
                    .names
                    .iter()
                    .position(|ws| *ws == name)
                    .unwrap();
                if delete {
                    let dev = home().join("Developer");
                    for provider in fs::read_dir(dev)? {
                        for user in fs::read_dir(provider?.path())? {
                            let user = user?;
                            for repo_or_workspace in fs::read_dir(&user.path())? {
                                let repo_or_workspace = repo_or_workspace?;
                                let name =
                                    repo_or_workspace.file_name().to_str().unwrap().to_string();
                                if settings.workspaces.names.contains(&name) {
                                    for repo in fs::read_dir(repo_or_workspace.path())? {
                                        let repo = repo?;
                                        fs_extra::dir::move_dir(
                                            repo.path(),
                                            &user.path(),
                                            &Default::default(),
                                        )?;
                                    }
                                    fs::remove_dir_all(repo_or_workspace.path())?;
                                }
                            }
                        }
                    }
                    settings.workspaces.names.remove(index);
                    settings.write()?;
                } else if rename.is_some() {
                    let dev = home().join("Developer");
                    for provider in fs::read_dir(dev)? {
                        for user in fs::read_dir(provider?.path())? {
                            let user = user?;
                            for repo_or_workspace in fs::read_dir(&user.path())? {
                                let repo_or_workspace = repo_or_workspace?;
                                let name =
                                    repo_or_workspace.file_name().to_str().unwrap().to_string();
                                if settings.workspaces.names.contains(&name) {
                                    fs::rename(
                                        repo_or_workspace.path(),
                                        repo_or_workspace
                                            .path()
                                            .parent()
                                            .unwrap()
                                            .join(rename.clone().unwrap()),
                                    )?;
                                }
                            }
                        }
                    }
                    *settings.workspaces.names.get_mut(index).unwrap() = rename.clone().unwrap();
                    settings.write()?;
                    println!("Workspace renamed to {}.", rename.unwrap());
                } else {
                    println!("Workspace `{name}` found.");
                }
            } else if delete || rename.is_some() {
                bail!("Couldn't find a workspace that matches {name}.")
            } else {
                settings.workspaces.names.push(name.clone());
                settings.write()?;
                println!("Workspace \"{name}\" was added.")
            }
        } else if list {
            let workspaces = settings.workspaces.names;
            println!("Currently available workspaces: {:?}", workspaces);
        }
        Ok(())
    }
}

fn get_settings() -> Result<Settings> {
    Settings::current()
        .with_context(|| APP_OPTIONS_NOT_FOUND)
}
