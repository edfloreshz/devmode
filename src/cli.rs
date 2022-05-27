use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use libset::config::Config;
use libset::format::FileFormat;
use regex::bytes::Regex;
use requestty::{Answer, Question};

use crate::config::application::Application;
use crate::config::editor::Editor;
use crate::config::fork::ForkAction;
use crate::config::host::Host;
use crate::config::project::OpenAction;
use crate::config::settings::Settings;
use crate::constants::messages::APP_OPTIONS_NOT_FOUND;
use crate::{config::clone::CloneAction, constants::patterns::GIT_URL};

#[derive(Parser, Debug)]
#[clap(name = "(Dev)mode", version = "0.2.8")]
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
        args: Vec<String>,
    },
    #[clap(about = "Opens a project on your selected text editor.", alias = "o")]
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
    #[clap(about = "Write changes to your configuration.", alias = "cf")]
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
    #[clap(about = "Create workspaces to store your projects.", alias = "ws")]
    Workspace {
        #[clap(help = "Delete a workspace", short = 'd', long = "delete")]
        delete: bool,
        #[clap(help = "Rename a workspace", short = 'r', long = "rename")]
        rename: bool,
        #[clap(help = "List all workspaces", short = 'l', long = "list")]
        list: bool
    }
}

impl Cli {
    pub fn run(&self) -> anyhow::Result<()> {
        let rx = Regex::new(GIT_URL).with_context(|| "Unable to parse Regex.")?;
        match &self.commands {
            Commands::Clone { args } => Cli::clone(args),
            Commands::Open { project } => Cli::open(project),
            Commands::Fork { args, upstream } => Cli::fork(args, upstream, rx),
            Commands::Config {
                map,
                show,
                all,
                editor,
                owner,
                host,
            } => Cli::config(*map, *show, *all, *editor, *owner, *host),
            Commands::Workspace { delete, rename, list } => {
                Cli::workspace(*delete, *rename, *list)
            },
        }
    }
    fn clone(args: &[String]) -> Result<()> {
        let clone = if args.is_empty() {
            clone_setup()?
        } else if args.len() == 1 {
            CloneAction::from_url(args.get(0).unwrap())?
        } else if args.len() == 3 {
            let host = Host::from(args.get(0).unwrap());
            let owner = args.get(1).unwrap();
            let repo = args.get(2).unwrap();
            CloneAction::from(host, owner, vec![repo.to_string()])
        } else {
            let options = Config::get::<Settings>("devmode/config/config.toml", FileFormat::TOML)
                .with_context(|| APP_OPTIONS_NOT_FOUND)?;
            CloneAction::from(Host::from(&options.host), &options.owner, args.to_vec())
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
            let options = Config::get::<Settings>("devmode/config/config.toml", FileFormat::TOML)
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
    ) -> Result<()> {
        if all {
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
    fn workspace(delete: bool, rename: bool, list: bool) -> Result<()> {
        if delete {
            todo!("Move repositories from workspace to user folder and delete the folder.")
        } else if rename {
            todo!("Rename the workspace folder and update the paths.")
        } else if list {
            todo!("List all the repositories from that workspace.")
        }
        Ok(())
    }
}

fn get_settings() -> Result<Settings> {
    Config::get::<Settings>("devmode/config/config.toml", FileFormat::TOML)
        .with_context(|| APP_OPTIONS_NOT_FOUND)
}

pub fn clone_setup() -> Result<CloneAction> {
    let mut clone = CloneAction::new();
    if let Answer::ListItem(host) = pick("host", "Choose your Git host:", vec!["GitHub", "GitLab"])?
    {
        clone.host = Host::from(&host.text);
    }
    if let Answer::String(owner) = ask("owner", "Git username:", "Please enter a Git username.")? {
        clone.owner = owner;
    }
    if let Answer::String(repo) = ask("repo", "Git repo name:", "Please enter a Git repo name.")? {
        clone.repos.push(repo);
    }
    Ok(clone)
}

pub fn fork_setup() -> Result<ForkAction> {
    let mut fork = ForkAction::new();
    if let Answer::ListItem(host) = pick("host", "Choose your Git host:", vec!["GitHub", "GitLab"])?
    {
        fork.host = Host::from(&host.text);
    }
    if let Answer::String(owner) = ask("owner", "Git username:", "Please enter a Git username.")? {
        fork.owner = owner;
    }
    if let Answer::String(repo) = ask("repo", "Git repo name:", "Please enter a Git repo name.")? {
        fork.repo = repo;
    }
    if let Answer::String(repo) = ask("upstream", "Upstream URL:", "Please enter an upstream URL.")?
    {
        fork.upstream = repo;
    }
    Ok(fork)
}

/// Runs the configuration setup again.
pub fn config_all() -> anyhow::Result<Settings> {
    let settings = Settings::new(
        config_host()?.host,
        config_owner()?.owner,
        config_editor()?.editor,
    );
    Ok(settings)
}

pub fn config_owner() -> anyhow::Result<Settings> {
    let answer = ask("owner", "Git username:", "Please enter a Git username.")?;
    let owner = match answer {
        Answer::String(owner) => owner,
        _ => bail!("Owner is required."),
    };
    let current = Config::get::<Settings>("devmode/config/config.toml", FileFormat::TOML);
    let settings = match current {
        None => Settings {
            owner,
            ..Default::default()
        },
        Some(mut settings) => {
            settings.owner = owner;
            settings
        }
    };
    Ok(settings)
}

pub fn config_host() -> anyhow::Result<Settings> {
    let answer = pick("host", "Choose your Git host:", vec!["GitHub", "GitLab"])?;
    let host = match answer {
        Answer::ListItem(item) => Host::from(&item.text).to_string(),
        _ => bail!("Host is required."),
    };
    let current = Config::get::<Settings>("devmode/config/config.toml", FileFormat::TOML);
    let settings = match current {
        None => Settings {
            host,
            ..Default::default()
        },
        Some(mut settings) => {
            settings.host = host;
            settings
        }
    };
    Ok(settings)
}

pub fn config_editor() -> anyhow::Result<Settings> {
    let answer = pick(
        "editor",
        "Choose your favorite editor:",
        vec!["Vim", "VSCode", "Custom"],
    )?;
    let editor = match answer {
        Answer::ListItem(item) => {
            if item.text.to_lowercase() == "custom" {
                let answer = ask(
                    "command",
                    "Editor command:",
                    "Please enter a editor command.",
                )?;
                if let Answer::String(name) = answer {
                    Editor::custom(name)
                } else {
                    bail!("Editor name is required.")
                }
            } else {
                Editor::new(Application::from(&*item.text))
            }
        }
        _ => bail!("Editor must be picked."),
    };
    let current = Config::get::<Settings>("devmode/config/config.toml", FileFormat::TOML);
    let settings = match current {
        None => Settings {
            editor,
            ..Default::default()
        },
        Some(mut settings) => {
            settings.editor = editor;
            settings
        }
    };
    Ok(settings)
}

pub fn select_repo(paths: Vec<&str>) -> anyhow::Result<String> {
    let answer = pick("repo", "Select the repository you want to open:", paths)?;
    let repo = match answer {
        Answer::ListItem(item) => item.text,
        _ => bail!("Repository must be picked."),
    };
    Ok(repo)
}

fn ask(key: &str, message: &str, err: &str) -> Result<Answer> {
    requestty::prompt_one(
        Question::input(key)
            .message(message)
            .validate(|owner, _previous| if owner.is_empty() {
                Err(err.into())
            } else {
                Ok(())
            })
            .build(),
    )
    .with_context(|| "Failed to present prompt.")
}

fn pick(key: &str, message: &str, options: Vec<&str>) -> anyhow::Result<Answer> {
    requestty::prompt_one(
        Question::select(key)
            .message(message)
            .choices(options)
            .build(),
    )
    .with_context(|| "Failed to get input.")
}
