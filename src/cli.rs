use crate::config::editor::Editor;
use crate::config::editor_app::EditorApp;
use crate::config::fork::Fork;
use crate::config::host::{is_host, Host};
use crate::config::project::Project;
use crate::config::settings::Settings;
use crate::constants::messages::APP_OPTIONS_NOT_FOUND;
use crate::{config::clone::CloneAction, constants::patterns::GIT_URL};
use anyhow::{Context, Result, bail};
use clap::{AppSettings, Parser, Subcommand};
use libdmd::config::Config;
use libdmd::format::FileType;
use regex::bytes::Regex;
use requestty::Answer;

#[derive(Parser, Debug)]
#[clap(name = "(Dev)mode", version = "0.2.5")]
#[clap(author = "Eduardo F. <edfloreshz@gmail.com>")]
#[clap(about = "Dev(mode) is a project management utility for developers.")]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
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
        #[clap(help = "Reconfigure everything.", short = 'a', long = "all")]
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
}

impl Cli {
    pub fn run(&self) -> anyhow::Result<()> {
        let rx = Regex::new(GIT_URL).with_context(|| "Unable to parse Regex.")?;
        match &self.commands {
            Commands::Clone { args } => {
                if args.is_empty() {
                    let clone = clone_setup()?;
                    clone.run()
                } else if rx.is_match(args.get(0).unwrap().as_bytes()) {
                    let clone = CloneAction::parse_url(args.get(0).unwrap(), rx)?;
                    clone.run()
                } else if is_host(args) {
                    let host = Host::from(args.get(0).unwrap());
                    let owner = args.get(1).with_context(|| "Failed to get owner.")?;
                    let repo = args.get(2).with_context(|| "Failed to get repository.")?;
                    let clone = CloneAction::from(host, owner, vec![repo.to_string()]);
                    clone.run()
                } else {
                    let options =
                        Config::get::<Settings>("devmode/config/config.toml", FileType::TOML)
                            .with_context(|| APP_OPTIONS_NOT_FOUND)?;
                    let clone =
                        CloneAction::from(Host::from(&options.host), &options.owner, args.to_vec());
                    clone.run()
                }
            }
            Commands::Open { project } => Project::new(project).run(),
            Commands::Fork { args, upstream } => {
                if args.is_empty() {
                    let fork = fork_setup()?;
                    fork.run()
                } else if rx.is_match(args.get(0).unwrap().as_bytes()) {
                    let fork = Fork::parse_url(args.get(0).unwrap(), rx, upstream.to_string())?;
                    fork.run()
                } else if args.len() == 1 {
                    let options =
                        Config::get::<Settings>("devmode/config/config.toml", FileType::TOML)
                            .with_context(|| APP_OPTIONS_NOT_FOUND)?;
                    let host = Host::from(&options.host);
                    let repo = args.get(0).map(|a| a.to_string());
                    let fork = Fork::from(
                        host,
                        upstream.to_string(),
                        options.owner,
                        repo.with_context(|| "Failed to get repo name.")?,
                    );
                    fork.run()
                } else {
                    let host = Host::from(args.get(0).unwrap());
                    let owner = args.get(1).map(|a| a.to_string());
                    let repo = args.get(2).map(|a| a.to_string());
                    let fork = Fork::from(
                        host,
                        upstream.to_string(),
                        owner.with_context(|| "Failed to get owner name.")?,
                        repo.with_context(|| "Failed to get repo name")?,
                    );
                    fork.run()
                }
            }
            Commands::Config {
                map,
                show,
                all,
                editor,
                owner,
                host,
            } => {
                if *all {
                    let settings = config_all()?;
                    settings.run()
                } else if *map {
                    Project::make_dev_paths()
                } else if get_settings().is_ok() {
                    if *editor {
                        let editor = config_editor().with_context(|| "Failed to set editor.")?;
                        editor.run()
                    } else if *owner {
                        let owner = config_owner().with_context(|| "Failed to set owner.")?;
                        owner.run()
                    } else if *host {
                        let host = config_host().with_context(|| "Failed to set host.")?;
                        host.run()
                    } else if *show {
                        let settings = get_settings()?;
                        settings.show();
                        Ok(())
                    } else {
                        let settings = get_settings()?;
                        settings.run()
                    }
                } else {
                    let settings = config_all()?;
                    settings.init()
                }
            }
        }
    }
}

fn get_settings() -> Result<Settings> {
    Config::get::<Settings>("devmode/config/config.toml", FileType::TOML)
        .with_context(|| APP_OPTIONS_NOT_FOUND)
}

pub fn clone_setup() -> Result<CloneAction> {
    let mut clone = CloneAction::new();
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
    if let Answer::ListItem(host) = requestty::prompt_one(question)? {
        clone.host = Host::from(&host.text);
    }
    let question = requestty::Question::input("owner")
        .message("Git username:")
        .validate(|owner, _previous| {
            if owner.is_empty() {
                Err("Please enter a Git username.".to_owned())
            } else {
                Ok(())
            }
        })
        .build();
    if let Answer::String(owner) = requestty::prompt_one(question)? {
        clone.owner = owner;
    }
    let question = requestty::Question::input("repo")
        .message("Git repo name:")
        .validate(|owner, _previous| {
            if owner.is_empty() {
                Err("Please enter a Git repo name.".to_owned())
            } else {
                Ok(())
            }
        })
        .build();
    if let Answer::String(repo) = requestty::prompt_one(question)? {
        clone.repos.push(repo);
    }
    Ok(clone)
}

pub fn fork_setup() -> Result<Fork> {
    let mut fork = Fork::new();
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
    if let Answer::ListItem(host) = requestty::prompt_one(question)? {
        fork.host = Host::from(&host.text);
    }
    let question = requestty::Question::input("owner")
        .message("Git username:")
        .validate(|owner, _previous| {
            if owner.is_empty() {
                Err("Please enter a Git username.".to_owned())
            } else {
                Ok(())
            }
        })
        .build();
    if let Answer::String(owner) = requestty::prompt_one(question)? {
        fork.owner = owner;
    }
    let question = requestty::Question::input("repo")
        .message("Git repo name:")
        .validate(|owner, _previous| {
            if owner.is_empty() {
                Err("Please enter a Git repo name.".to_owned())
            } else {
                Ok(())
            }
        })
        .build();
    if let Answer::String(repo) = requestty::prompt_one(question)? {
        fork.repo = repo;
    }

    let question = requestty::Question::input("upstream")
        .message("Git URL (upstream):")
        .validate(|owner, _previous| {
            if owner.is_empty() {
                Err("Please enter a Git URL.".to_owned())
            } else {
                Ok(())
            }
        })
        .build();
    if let Answer::String(repo) = requestty::prompt_one(question)? {
        fork.upstream = repo;
    }
    Ok(fork)
}

/// Runs the configuration setup again.
pub fn config_all() -> anyhow::Result<Settings> {
    let editor = config_editor()?;
    let owner = config_owner()?;
    let host = config_host()?;
    let settings = Settings::new(host.host, owner.owner, editor.editor);
    Ok(settings)
}

pub fn config_owner() -> anyhow::Result<Settings> {
    let question = requestty::Question::input("owner")
        .message("What's your Git username:")
        .validate(|owner, _previous| {
            if owner.is_empty() {
                Err("Please enter a Git username.".to_owned())
            } else {
                Ok(())
            }
        })
        .build();
    let answer = requestty::prompt_one(question)?;
    let owner = if let Answer::String(owner) = answer {
        owner
    } else {
        bail!("Owner is required.")
    };
    let settings = if Config::get::<Settings>("devmode/config/config.toml", FileType::TOML).is_some() {
        let mut options = Config::get::<Settings>("devmode/config/config.toml", FileType::TOML).with_context(|| "Failed to get current settings.")?;
        options.owner = owner;
        options
    } else {
        Settings {
            owner,
            ..Default::default()
        }
    };
    Ok(settings)
}

pub fn config_host() -> anyhow::Result<Settings> {
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
        let answer = requestty::prompt_one(question)?;
    let host = if let Answer::ListItem(host) = answer {
        Host::from(&host.text).to_string()
    } else {
        bail!("Host is required.")
    };
    let settings = if Config::get::<Settings>("devmode/config/config.toml", FileType::TOML).is_some() {
        let mut options = Config::get::<Settings>("devmode/config/config.toml", FileType::TOML).with_context(|| "Failed to get current settings.")?;
        options.host = host;
        options
    } else {
        Settings {
            host,
            ..Default::default()
        }
    };
    Ok(settings)
}

pub fn config_editor() -> anyhow::Result<Settings> {
    let question = requestty::Question::select("editor")
        .message("Choose your favorite editor:")
        .choices(vec!["Vim", "VSCode", "Custom"])
        .build();
    let answer = requestty::prompt_one(question)?;
    let editor = if let Answer::ListItem(i) = answer {
        if i.text.to_lowercase() == "custom" {
            let question = requestty::Question::input("command")
                .message("Editor command:")
                .validate(|owner, _previous| {
                    if owner.is_empty() {
                        Err("Please enter a editor command".to_owned())
                    } else {
                        Ok(())
                    }
                })
                .build();
            let answer = requestty::prompt_one(question)?;
            if let Answer::String(name) = answer {
                Editor::custom(name)
            } else {
                bail!("Editor name is required.")
            }
        } else {
            Editor::new(EditorApp::from(&*i.text))
        }
    } else {
        bail!("Editor must be picked.")
    };
    let settings = if Config::get::<Settings>("devmode/config/config.toml", FileType::TOML).is_some() {
        let mut options = Config::get::<Settings>("devmode/config/config.toml", FileType::TOML).with_context(|| "Failed to get current settings.")?;
        options.editor = editor;
        options
    } else {
        Settings {
            editor,
            ..Default::default()
        }
    };
    Ok(settings)
}
