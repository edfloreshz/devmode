use std::{fs::remove_dir_all, path::PathBuf};

use devmode::clone::CloneAction;
use devmode::editor::Editor;
use devmode::fork::ForkAction;
use devmode::host::Host;
use devmode::settings::Settings;
use devmode::{application::Application, Error};
use requestty::{Answer, Question};
use url_builder::URLBuilder;

pub fn overwrite(path: PathBuf) -> Result<bool, Error> {
    println!(
        "Error: {} exists and is not an empty directory",
        path.display()
    );
    let question = requestty::Question::confirm("overwrite")
        .message("Do you want to overwrite the existing repository?")
        .build();
    let answer = requestty::prompt_one(question)?;
    if let requestty::Answer::Bool(overwrite) = answer {
        if overwrite {
            remove_dir_all(&path)?;
            return Ok(overwrite);
        }
    }
    Ok(false)
}

pub fn clone_setup() -> Result<CloneAction, Error> {
    let mut url = URLBuilder::new();
    url.set_protocol("https");
    if let Answer::ListItem(host) = pick("host", "Choose your Git host:", vec!["GitHub", "GitLab"])?
    {
        url.set_host(Host::from(&host.text).url());
    }
    if let Answer::String(owner) = ask("owner", "Git username:", "Please enter a Git username.")? {
        url.add_route(&owner);
    }
    if let Answer::String(repo) = ask("repo", "Git repo name:", "Please enter a Git repo name.")? {
        url.add_route(&repo);
    }

    let mut clone = CloneAction::new(&url.build());

    let settings = Settings::current().ok_or(Error::Generic("Failed to get configuration"))?;
    let mut options: Vec<&str> = settings
        .workspaces
        .names
        .iter()
        .map(|s| s.as_str())
        .collect();
    options.insert(0, "None");
    if let Answer::ListItem(workspace) = pick("workspace", "Pick a workspace", options)? {
        let workspace = workspace.text.to_lowercase();
        if !workspace.eq("none") {
            clone.set_workspace(workspace);
        }
    }
    Ok(clone)
}

pub fn fork_setup() -> Result<ForkAction, Error> {
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
pub fn config_all() -> Result<Settings, Error> {
    let settings = Settings::new(
        config_host()?.host,
        config_owner()?.owner,
        config_editor()?.editor,
    );
    Ok(settings)
}

pub fn config_owner() -> Result<Settings, Error> {
    let answer = ask("owner", "Git username:", "Please enter a Git username.")?;
    let owner = match answer {
        Answer::String(owner) => owner,
        _ => return devmode::error("Owner is required."),
    };
    let current = Settings::current();
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

pub fn config_host() -> Result<Settings, Error> {
    let answer = pick("host", "Choose your Git host:", vec!["GitHub", "GitLab"])?;
    let host = match answer {
        Answer::ListItem(item) => Host::from(&item.text).to_string(),
        _ => return devmode::error("Host is required."),
    };
    let current = Settings::current();
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

pub fn config_editor() -> Result<Settings, Error> {
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
                    return devmode::error("Editor name is required.");
                }
            } else {
                Editor::new(Application::from(&*item.text))
            }
        }
        _ => return devmode::error("Editor must be picked."),
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

pub fn select_repo(paths: Vec<&str>) -> Result<String, Error> {
    let answer = pick("repo", "Select the repository you want to open:", paths)?;
    let repo = match answer {
        Answer::ListItem(item) => item.text,
        _ => return devmode::error("Repository must be picked."),
    };
    Ok(repo)
}

pub fn ask(key: &str, message: &str, err: &str) -> Result<Answer, Error> {
    requestty::prompt_one(
        Question::input(key)
            .message(message)
            .validate(|owner, _previous| {
                if owner.is_empty() {
                    Err(err.into())
                } else {
                    Ok(())
                }
            })
            .build(),
    )
    .map_err(|e| Error::String(e.to_string()))
}

pub fn pick(key: &str, message: &str, options: Vec<&str>) -> Result<Answer, Error> {
    requestty::prompt_one(
        Question::select(key)
            .message(message)
            .choices(options)
            .build(),
    )
    .map_err(|e| Error::String(e.to_string()))
}
