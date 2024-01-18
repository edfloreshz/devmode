use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use fs_extra::{dir, move_items};
use libset::routes::home;
use regex::bytes::Regex;
use requestty::Answer;
use std::fs;
use std::path::PathBuf;

use devmode_shared::constants::messages::*;

use crate::input::{
    clone_setup, config_all, config_editor, config_host, config_owner, fork_setup, select_repo,
};
use devmode_shared::fork::ForkAction;
use devmode_shared::host::Host;
use devmode_shared::project::{create_paths_reader, find_paths, OpenAction};
use devmode_shared::settings::Settings;
use devmode_shared::{clone::CloneAction, constants::patterns::GIT_URL};

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
        #[clap(
            help = "Add a repo to a workspace",
            short = 'a',
            long = "add",
            takes_value = true
        )]
        add: Option<String>,
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
    pub fn run(&self) -> Result<()> {
        let rx = Regex::new(GIT_URL).with_context(|| "Unable to parse Regex.")?;
        match &self.commands {
            Commands::Clone { args, workspace } => Cli::clone(args, workspace.to_owned()),
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
                add,
                remove,
                list,
            } => Cli::workspace(
                name.to_owned(),
                *delete,
                rename.clone(),
                add.clone(),
                remove.clone(),
                *list,
            ),
        }
    }
    fn clone(args: &[String], workspace: Option<String>) -> Result<()> {
        let clone = CloneAction::new().set_workspace(workspace);
        let mut clone = if args.is_empty() {
            clone_setup()?
        } else if args.len() == 1 && args.get(0).unwrap().contains("http") {
            clone.set_url(args.get(0).cloned())
        } else if args.len() == 3 {
            clone
                .set_host(Some(Host::from(args.get(0).unwrap())))
                .set_owner(args.get(1))
                .set_repos(Some(vec![args.get(2).unwrap().to_owned()]))
        } else {
            let options = Settings::current().with_context(|| APP_OPTIONS_NOT_FOUND)?;
            clone
                .set_host(Some(Host::from(&options.host)))
                .set_owner(Some(&options.owner))
                .set_repos(Some(args.to_vec()))
        };
        clone.run()
    }
    fn open(project: &str) -> Result<()> {
        let reader = create_paths_reader()?;
        let paths = find_paths(reader, project)?;
        if paths.is_empty() {
            bail!(NO_PROJECT_FOUND)
        } else if paths.len() > 1 {
            let paths: Vec<&str> = paths.iter().map(|s| s as &str).collect();
            let path = select_repo(paths)
                .with_context(|| "Failed to set repository.")?
                .to_string();
            OpenAction::new(project).open(vec![path])
        } else {
            OpenAction::new(project).open(paths)
        }
    }
    fn update(project: &str) -> Result<()> {
        let reader = create_paths_reader()?;
        let paths = find_paths(reader, project)?;
        if paths.is_empty() {
            bail!(NO_PROJECT_FOUND)
        } else if paths.len() > 1 {
            eprintln!("{}", MORE_PROJECTS_FOUND); // TODO: Let user decide which
            let paths: Vec<&str> = paths.iter().map(|s| s as &str).collect();
            let path = select_repo(paths).with_context(|| "Failed to set repository.")?;
            OpenAction::new(project).update(vec![path])
        } else {
            OpenAction::new(project).update(paths)
        }
    }
    fn fork(args: &[String], upstream: &str, rx: Regex) -> Result<()> {
        let action = if args.is_empty() {
            fork_setup()?
        } else if rx.is_match(args.get(0).unwrap().as_bytes()) {
            ForkAction::parse_url(args.get(0).unwrap(), rx, upstream.to_string())?
        } else if args.len() == 1 {
            let options = Settings::current().with_context(|| APP_OPTIONS_NOT_FOUND)?;
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
            settings.write(false)?;
        }
        if map {
            OpenAction::make_dev_paths()?
        }
        if editor {
            let settings = config_editor().with_context(|| "Failed to set editor.")?;
            settings.write(false)?
        }
        if owner {
            let settings = config_owner().with_context(|| "Failed to set owner.")?;
            settings.write(false)?
        }
        if host {
            let settings = config_host().with_context(|| "Failed to set host.")?;
            settings.write(false)?
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
        add: Option<String>,
        remove: Option<String>,
        list: bool,
    ) -> Result<()> {
        let mut settings = Settings::current().with_context(|| "Failed to get configuration")?;
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
                                let repo_name =
                                    repo_or_workspace.file_name().to_str().unwrap().to_string();
                                if settings.workspaces.names.contains(&repo_name)
                                    && repo_name.eq(&name)
                                {
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
                    settings.write(true)?;
                    println!(
                        "Workspace {} was successfully deleted.",
                        Colorize::yellow(&*name)
                    )
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
                    settings.write(true)?;
                    println!(
                        "Workspace renamed from {} to {}.",
                        Colorize::yellow(&*name),
                        Colorize::blue(&*rename.unwrap())
                    );
                } else if let Some(add) = add {
                    let reader = create_paths_reader()?;
                    let paths: Vec<String> = find_paths(reader, &add)?
                        .iter()
                        .map(|path| path.to_owned())
                        .filter(|path| !path.contains(name.as_str()))
                        .collect();
                    let path = if paths.is_empty() {
                        bail!("Could not locate the {add} repository.")
                    } else if paths.len() > 1 {
                        eprintln!("{}", MORE_PROJECTS_FOUND);
                        let paths: Vec<&str> = paths.iter().map(|s| s as &str).collect();
                        select_repo(paths).with_context(|| "Failed to set repository.")?
                    } else {
                        paths[0].clone()
                    };
                    let mut options = dir::CopyOptions::new();
                    let to = PathBuf::from(&path).parent().unwrap().join(&name);
                    if to.join(add.as_str()).exists() {
                        let question = requestty::Question::confirm("overwrite")
                            .message("We found an existing repository with the same name, do you want to overwrite the existing repository?")
                            .build();
                        let answer = requestty::prompt_one(question)?;
                        if let Answer::Bool(overwrite) = answer {
                            if overwrite {
                                options.overwrite = true;
                                move_items(&[path], to, &options)?;
                            }
                        }
                    } else {
                        move_items(&[path], to, &options)?;
                    }
                } else if let Some(remove) = remove {
                    let reader = create_paths_reader()?;
                    let paths: Vec<String> = find_paths(reader, &remove)?
                        .iter()
                        .map(|path| path.to_owned())
                        .filter(|path| path.contains(name.as_str()))
                        .collect();
                    let path = if paths.is_empty() {
                        bail!("Could not locate the {remove} repository inside {name}")
                    } else if paths.len() > 1 {
                        eprintln!("{}", MORE_PROJECTS_FOUND);
                        let paths: Vec<&str> = paths.iter().map(|s| s as &str).collect();
                        select_repo(paths).with_context(|| "Failed to set repository.")?
                    } else {
                        paths[0].clone()
                    };
                    let mut options = dir::CopyOptions::new();
                    let path = PathBuf::from(&path);
                    let to = path.parent().unwrap().parent().unwrap();
                    if to.join(remove.as_str()).exists() {
                        let question = requestty::Question::confirm("overwrite")
                            .message("We found an existing repository with the same name, do you want to overwrite the existing repository?")
                            .build();
                        let answer = requestty::prompt_one(question)?;
                        if let Answer::Bool(overwrite) = answer {
                            if overwrite {
                                options.overwrite = true;
                                move_items(&[path.clone()], to, &options)?;
                            }
                        }
                    } else {
                        move_items(&[path.clone()], to, &options)?;
                    }
                } else {
                    println!("Workspace `{}` found.", Colorize::green(&*name));
                }
            } else if delete || rename.is_some() {
                bail!(
                    "Couldn't find a workspace that matches {}.",
                    Colorize::yellow(&*name)
                )
            } else {
                settings.workspaces.names.push(name.clone());
                settings.write(true)?;
                println!("Workspace {} was added.", Colorize::yellow(&*name))
            }
        } else if list {
            let workspaces = settings.workspaces.names;
            println!(
                "Currently available workspaces: {}",
                Colorize::yellow(&*format!("{:?}", workspaces))
            );
        }
        Ok(())
    }
}

fn get_settings() -> Result<Settings> {
    Settings::current().with_context(|| APP_OPTIONS_NOT_FOUND)
}
