use clap::{Parser, Subcommand};
use devmode::action::Action;
use devmode::config::Config;
use devmode::workspace::{Workspace, WorkspaceOptions};
use devmode::{DevmodeError, Error};
use fs_extra::dir::CopyOptions;
use fs_extra::{dir, move_items};
use regex::bytes::Regex;
use std::fs::remove_dir_all;
use std::path::PathBuf;
use url_builder::URLBuilder;

use crate::input::{
    clone_setup, config_all, config_editor, config_host, config_owner, create_workspace,
    fork_setup, overwrite, select_repo,
};
use devmode::fork::ForkAction;
use devmode::host::Host;
use devmode::project::{find_paths, OpenAction};
use devmode::settings::Settings;
use devmode::{clone::CloneAction, constants::patterns::GIT_URL};

#[derive(Parser, Debug)]
#[clap(name = "(Dev)mode", version = "0.3.0")]
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
        about = "Fetch the latest changes for a project.",
        alias = "u",
        arg_required_else_help = true
    )]
    Update {
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
        #[clap(help = "Configure your settings.", short = 'a', long = "all")]
        all: bool,
        #[clap(help = "Set preferred code editor.", short = 'e', long = "editor")]
        editor: bool,
        #[clap(help = "Set preferred git username.", short = 'o', long = "owner")]
        owner: bool,
        #[clap(help = "Set preferred Git host to clone projects from.", long = "host")]
        host: bool,
    },
    #[clap(
        about = "Create workspaces to store your projects.",
        alias = "ws",
        arg_required_else_help = true
    )]
    Workspace {
        #[clap(help = "The name of the workspace")]
        name: Option<String>,
        #[clap(help = "Add a workspace", short = 'a', long = "add")]
        add: bool,
        #[clap(help = "Delete a workspace", short = 'd', long = "delete")]
        delete: bool,
        #[clap(
            help = "Rename a workspace",
            short = 'r',
            long = "rename",
            takes_value = true
        )]
        rename: Option<String>,
        #[clap(
            help = "Include a repo in a workspace",
            short = 'i',
            long = "include",
            takes_value = true
        )]
        include: Option<String>,
        #[clap(
            help = "Remove a repo from a workspace",
            short = 'm',
            long = "remove",
            takes_value = true
        )]
        remove: Option<String>,
        #[clap(help = "List all workspaces", short = 'l', long = "list")]
        list: bool,
    },
}

impl Cli {
    pub fn run(&self) -> Result<(), Error> {
        let rx = Regex::new(GIT_URL)?;
        match &self.commands {
            Commands::Clone { args, workspace } => Cli::clone(args.clone(), workspace.to_owned()),
            Commands::Open { project } => Cli::open(project),
            Commands::Update { project } => Cli::update(project),
            Commands::Fork { args, upstream } => Cli::fork(args, upstream, rx),
            Commands::Config {
                map,
                show,
                all,
                editor,
                owner,
                host,
            } => Cli::config(Config {
                map: *map,
                show: *show,
                all: *all,
                editor: *editor,
                owner: *owner,
                host: *host,
                none: !map && !show && !all && !editor && !owner && !host,
            }),
            Commands::Workspace {
                name,
                add,
                delete,
                rename,
                include,
                remove,
                list,
            } => Cli::workspace(WorkspaceOptions {
                name: name.to_owned(),
                add: *add,
                delete: *delete,
                rename: rename.to_owned(),
                include: include.to_owned(),
                remove: remove.to_owned(),
                list: *list,
            }),
        }
    }

    fn clone(args: Vec<String>, workspace: Option<String>) -> Result<(), Error> {
        let mut url = URLBuilder::new();
        url.set_protocol("https");
        let mut clone = if args.is_empty() {
            clone_setup()?
        } else if Settings::current().is_some() && args.len().eq(&1) {
            let Some(options) = Settings::current() else {
                return Err(Error::Devmode(DevmodeError::AppSettingsNotFound));
            };

            url.set_host(Host::from(&options.host).url())
                .add_route(&options.owner);
            if let Some(repo) = args.first() {
                url.add_route(repo);
            }

            CloneAction::new(&url.build())
        } else if args.len().eq(&1) {
            if let Some(url) = args.first() {
                CloneAction::new(url)
            } else {
                return Err(Error::Devmode(DevmodeError::NoUrlProvided));
            }
        } else if args.len().eq(&3) {
            if let Some(host) = args.first() {
                url.set_host(Host::from(host).url());
            }
            if let Some(owner) = args.get(1) {
                url.add_route(owner);
            }
            if let Some(repo) = args.get(2) {
                url.add_route(repo);
            }
            CloneAction::new(&url.build())
        } else {
            return Err(Error::Devmode(DevmodeError::InvalidCommand));
        };
        if let Some(workspace) = workspace {
            clone.set_workspace(workspace);
        }

        if let Err(error) = clone.run() {
            match error {
                Error::Git(error) => match error.code() {
                    git2::ErrorCode::Exists => {
                        let path = clone.get_local_path()?;
                        println!(
                            "Error: {} exists and is not an empty directory",
                            path.display()
                        );
                        if overwrite()? {
                            remove_dir_all(&path)?;
                            clone.run()?;
                        }
                    }
                    _ => log::error!("{error}"),
                },
                error => log::error!("{error}"),
            }
        };

        Ok(())
    }

    fn open(project: &str) -> Result<(), Error> {
        let paths = find_paths(project)?;
        if paths.is_empty() {
            Err(Error::Devmode(DevmodeError::NoProjectFound))
        } else if paths.len() > 1 {
            let path = select_repo(project, None)?;
            OpenAction::new(project).open(path)
        } else {
            OpenAction::new(project).open(
                paths
                    .get(0)
                    .ok_or(Error::Devmode(DevmodeError::PathNotFound))?
                    .clone(),
            )
        }
    }

    fn update(project: &str) -> Result<(), Error> {
        let paths = find_paths(project)?;
        if paths.is_empty() {
            Err(Error::Devmode(DevmodeError::NoProjectFound))
        } else if paths.len() > 1 {
            let path = select_repo(project, None)?;
            OpenAction::new(project).update(path)
        } else {
            OpenAction::new(project).update(
                paths
                    .get(0)
                    .ok_or(Error::Devmode(DevmodeError::PathNotFound))?
                    .clone(),
            )
        }
    }

    fn fork(args: &[String], upstream: &str, rx: Regex) -> Result<(), Error> {
        let action = if args.is_empty() {
            fork_setup()?
        } else if rx.is_match(args.first().unwrap().as_bytes()) {
            ForkAction::parse_url(args.first().unwrap(), rx, upstream.to_string())?
        } else if args.len().eq(&1) {
            let options =
                Settings::current().ok_or(Error::Devmode(DevmodeError::AppSettingsNotFound))?;
            let host = Host::from(&options.host);
            let repo = args
                .first()
                .map(|a| a.to_string())
                .ok_or(Error::Generic("Failed to get repo"))?;
            ForkAction::from(host, upstream.to_string(), options.owner, repo)
        } else {
            let host = Host::from(args.first().unwrap());
            let owner = args
                .get(1)
                .map(|a| a.to_string())
                .ok_or(Error::Generic("Failed to get owner"))?;
            let repo = args
                .get(2)
                .map(|a| a.to_string())
                .ok_or(Error::Generic("Failed to get repo"))?;
            ForkAction::from(host, upstream.to_string(), owner, repo)
        };
        action.run()
    }
    fn config(config: Config) -> Result<(), Error> {
        if config.all || config.none {
            if get_settings().is_err() {
                println!("First time setup! 🥳\n");
                Settings::init()?;
            }
            let settings = config_all()?;
            settings.write(false)?;
        }
        if config.map {
            OpenAction::make_dev_paths()?
        }
        if config.editor {
            let settings = config_editor()?;
            settings.write(false)?
        }
        if config.owner {
            let settings = config_owner()?;
            settings.write(false)?
        }
        if config.host {
            let settings = config_host()?;
            settings.write(false)?
        }
        if config.show {
            let settings = get_settings()?;
            settings.show();
        }
        Ok(())
    }

    fn workspace(arguments: WorkspaceOptions) -> Result<(), Error> {
        let mut settings =
            Settings::current().ok_or(Error::Devmode(DevmodeError::AppSettingsNotFound))?;
        let Some(ref workspace_name) = arguments.name else {
            let workspaces = settings.workspaces.names;
            println!("Currently available workspaces: {workspaces:?}");
            return Ok(());
        };
        let mut workspace = Workspace::new(&workspace_name);
        if settings.workspaces.names.contains(workspace_name) {
            if arguments.delete {
                workspace.delete()?;
                println!("Workspace {workspace_name} was successfully deleted.");
            } else if let Some(ref to) = arguments.rename {
                workspace.rename(to)?;
                println!("Workspace renamed from {workspace_name} to {to}.");
            } else if let Some(ref project) = arguments.include {
                let paths: Vec<PathBuf> = find_paths(project)?
                    .iter()
                    .filter(|path| !path.display().to_string().contains(workspace_name))
                    .map(PathBuf::from)
                    .collect();
                let project = if paths.len() > 0 {
                    select_repo(project, Some(workspace_name))?
                } else {
                    paths
                        .get(0)
                        .ok_or(Error::Devmode(DevmodeError::ProjectNotFound))?
                        .clone()
                };
                let mut options = CopyOptions::new();
                let destination = project
                    .parent()
                    .ok_or(Error::Devmode(DevmodeError::PathNotFound))?
                    .join(&workspace_name);

                if destination.exists() {
                    options.overwrite = overwrite()?;
                }

                move_items(&[project], destination, &options)?;
            } else if let Some(ref project_name) = arguments.remove {
                let paths: Vec<PathBuf> = find_paths(project_name)?
                    .iter()
                    .filter(|path| !path.display().to_string().contains(workspace_name))
                    .map(PathBuf::from)
                    .collect();
                let project = if paths.len() > 0 {
                    select_repo(project_name, Some(workspace_name))?
                } else {
                    paths
                        .get(0)
                        .ok_or(Error::Devmode(DevmodeError::ProjectNotFound))?
                        .clone()
                };
                let mut options = dir::CopyOptions::new();
                let to = project
                    .parent()
                    .ok_or(Error::Devmode(DevmodeError::PathNotFound))?
                    .parent()
                    .ok_or(Error::Devmode(DevmodeError::PathNotFound))?;

                if to.join(&project).exists() {
                    options.overwrite = overwrite()?;
                }

                move_items(&[project.clone()], to, &options)?;
            } else {
                println!("Workspace `{workspace_name}` found.");
            }
        } else if arguments.delete || arguments.rename.is_some() {
            return devmode::error("Couldn't find a workspace that matches {name}.");
        } else if arguments.add {
            settings.workspaces.names.push(workspace_name.clone());
            settings.write(true)?;
            println!("Workspace {workspace_name} was added.")
        } else {
            let create = create_workspace()?;
            if create {
                settings.workspaces.names.push(workspace_name.clone());
                settings.write(true)?;
                println!("Workspace {workspace_name} was added.")
            }
        }
        Ok(())
    }
}

fn get_settings() -> Result<Settings, Error> {
    Settings::current().ok_or(Error::Devmode(DevmodeError::AppSettingsNotFound))
}
