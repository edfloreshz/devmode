use std::path::PathBuf;

use devmode::clone::CloneAction;
use devmode::constants::names::{CUSTOM_NAME, NONE, VIM_NAME, VSCODE_NAME};
use devmode::editor::Editor;
use devmode::fork::ForkAction;
use devmode::host::Host;
use devmode::settings::Settings;
use devmode::DevmodeError;
use devmode::{application::Application, Error};
use requestty::{Answer, Question};
use url_builder::URLBuilder;

pub fn confirm(message: &str, id: &str) -> Result<bool, Error> {
    let question = requestty::Question::confirm(id).message(message).build();
    let answer = requestty::prompt_one(question)?;
    if let Answer::Bool(confirm) = answer {
        Ok(confirm)
    } else {
        Err(Error::Unknown)
    }
}

pub fn input(key: &str, message: &str, err: &str) -> Result<String, Error> {
    let question = Question::input(key)
        .message(message)
        .validate(|answer, _| {
            if answer.is_empty() {
                Err(err.into())
            } else {
                Ok(())
            }
        })
        .build();
    let answer = requestty::prompt_one(question)?;
    if let Answer::String(output) = answer {
        Ok(output)
    } else {
        Err(Error::Unknown)
    }
}

pub fn select(key: &str, message: &str, options: Vec<impl Into<String>>) -> Result<String, Error> {
    let question = Question::select(key)
        .message(message)
        .choices(options)
        .build();
    let answer = requestty::prompt_one(question)?;
    if let Answer::ListItem(item) = answer {
        Ok(item.text)
    } else {
        Err(Error::Unknown)
    }
}

pub fn clone_setup() -> Result<CloneAction, Error> {
    let mut url = URLBuilder::new();
    url.set_protocol("https");
    let host = select("host", "Choose your Git host:", vec!["GitHub", "GitLab"])?;
    url.set_host(Host::from(&host).url());
    let owner = input("owner", "Git username:", "Please enter a Git username.")?;
    url.add_route(&owner);
    let repo = input("repo", "Git repo name:", "Please enter a Git repo name.")?;
    url.add_route(&repo);

    let mut clone = CloneAction::new(&url.build());

    let settings = Settings::current().ok_or(Error::Devmode(DevmodeError::AppSettingsNotFound))?;
    let mut options: Vec<&str> = settings
        .workspaces
        .names
        .iter()
        .map(|s| s.as_str())
        .collect();
    options.insert(0, NONE);
    let workspace = select("workspace", "Pick a workspace", options)?;
    if !workspace.eq(NONE) {
        clone.set_workspace(workspace);
    }
    Ok(clone)
}

pub fn fork_setup() -> Result<ForkAction, Error> {
    let mut fork = ForkAction::new();
    let host = select("host", "Choose your Git host:", vec!["GitHub", "GitLab"])?;
    fork.host = Host::from(&host);
    let owner = input("owner", "Git username:", "Please enter a Git username.")?;
    fork.owner = owner;
    let repo = input("repo", "Git repo name:", "Please enter a Git repo name.")?;
    fork.repo = repo;
    let repo = input("upstream", "Upstream URL:", "Please enter an upstream URL.")?;
    fork.upstream = repo;
    Ok(fork)
}

/// Runs the configuration setup again.
pub fn config_all() -> Result<Settings, Error> {
    let settings = Settings::new(
        config_host()?.host,
        config_owner()?.owner,
        config_editor()?.editor,
    );
    Ok(settings)
}

pub fn config_owner() -> Result<Settings, Error> {
    let owner = input("owner", "Git username:", "Please enter a Git username.")?;
    let mut settings = Settings::current().unwrap_or_default();
    settings.owner = owner;
    Ok(settings)
}

pub fn config_host() -> Result<Settings, Error> {
    let host = select("host", "Choose your Git host:", vec!["GitHub", "GitLab"])?;
    let mut settings = Settings::current().unwrap_or_default();
    settings.host = host;
    Ok(settings)
}

pub fn config_editor() -> Result<Settings, Error> {
    let editor = select(
        "editor",
        "Choose your favorite editor:",
        vec![VIM_NAME, VSCODE_NAME, CUSTOM_NAME],
    )?;
    let editor = if editor.eq(CUSTOM_NAME) {
        let command = input(
            "command",
            "Editor command:",
            "Please enter a editor command.",
        )?;
        Editor::custom(command)
    } else {
        Editor::new(Application::from(&editor))
    };
    let current = Settings::current();
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

pub fn select_repo(paths: Vec<PathBuf>) -> Result<PathBuf, Error> {
    let paths: Vec<String> = paths.iter().map(|s| s.display().to_string()).collect();
    let repo = if paths.len() > 1 {
        select("repo", "Select the repository you want to open:", paths)?
    } else {
        paths[0].clone()
    };

    Ok(PathBuf::from(repo))
}

pub fn create_workspace() -> Result<bool, Error> {
    let create = confirm("Would you like to create this workspace?", "workspace")?;
    Ok(create)
}

pub fn overwrite() -> Result<bool, Error> {
    let overwrite = confirm("Found existing repository, overwrite it?", "overwrite")?;
    Ok(overwrite)
}
