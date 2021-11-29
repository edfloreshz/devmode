use crate::config::clone::CloneAction;
use crate::config::editor::Editor;
use crate::config::editor_app::EditorApp;
use crate::config::fork::Fork;
use crate::config::host::Host;
use crate::config::settings::Settings;
use anyhow::Result;
use libdmd::utils::config::config::Config;
use libdmd::utils::config::format::FileFormat::TOML;
use requestty::Answer;

use crate::cmd::*;

pub fn clone_setup() -> Result<Cmd> {
    let mut clone = CloneAction::new();
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
    if let Answer::ListItem(host) = requestty::prompt_one(question)? {
        clone.host = Host::from(host.text);
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
    Ok(Cmd::Clone(clone))
}

pub fn fork_setup() -> Result<Cmd> {
    let mut fork = Fork::new();
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
    if let Answer::ListItem(host) = requestty::prompt_one(question)? {
        fork.host = Host::from(host.text);
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
    Ok(Cmd::Fork(fork))
}

pub fn config_all() -> Option<Cmd> {
    let editor = config_editor();
    let editor = if let Some(Cmd::Config(options)) = editor {
        options.editor
    } else {
        Editor::default()
    };
    let owner = config_owner();
    let owner = if let Some(Cmd::Config(options)) = owner {
        options.owner
    } else {
        String::new()
    };
    let host = config_host();
    let host = if let Some(Cmd::Config(options)) = host {
        options.host
    } else {
        String::new()
    };
    Some(Cmd::Config(Settings::new(host, owner, editor)))
}

pub fn config_owner() -> Option<Cmd> {
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
    let owner = if let Answer::String(owner) = requestty::prompt_one(question).ok()? {
        owner
    } else {
        String::new()
    };
    if Config::get::<Settings>("devmode/config/config.toml", TOML).is_some() {
        let mut options = Config::get::<Settings>("devmode/config/config.toml", TOML).unwrap();
        options.owner = owner;
        Some(Cmd::Config(options))
    } else {
        Some(Cmd::Config(Settings::new(
            String::new(),
            owner,
            Default::default(),
        )))
    }
}

pub fn config_host() -> Option<Cmd> {
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
    let host = if let Answer::ListItem(host) = requestty::prompt_one(question).ok()? {
        Host::from(host.text).to_string()
    } else {
        Host::None.to_string()
    };
    if Config::get::<Settings>("devmode/config/config.toml", TOML).is_some() {
        let mut options = Config::get::<Settings>("devmode/config/config.toml", TOML).unwrap();
        options.host = host;
        Some(Cmd::Config(options))
    } else {
        Some(Cmd::Config(Settings::new(
            host,
            String::new(),
            Default::default(),
        )))
    }
}

pub fn config_editor() -> Option<Cmd> {
    let question = requestty::Question::select("editor")
        .message("Choose your favorite editor:")
        .choices(vec!["Vim", "VSCode", "Custom"])
        .build();
    let editor = if let Answer::ListItem(i) = requestty::prompt_one(question).ok()? {
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
            if let Answer::String(cmd) = requestty::prompt_one(question).ok()? {
                let command = Option::from(cmd);
                Some(Editor::custom(command.unwrap()))
            } else {
                None
            }
        } else {
            Some(Editor::new(EditorApp::from(&*i.text)))
        }
    } else {
        None
    };
    if Config::get::<Settings>("devmode/config/config.toml", TOML).is_some() && editor.is_some() {
        let mut options = Config::get::<Settings>("devmode/config/config.toml", TOML).unwrap();
        options.editor = editor?;
        Some(Cmd::Config(options))
    } else {
        if editor.is_some() {
            Some(Cmd::Config(Settings::new(
                "".to_string(),
                "".to_string(),
                editor?,
            )))
        } else {
            None
        }
    }
}
