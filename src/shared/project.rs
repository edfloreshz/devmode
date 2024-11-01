use std::path::PathBuf;
use std::process::Command;

use cmd_lib::*;
use libset::routes::home;
use walkdir::{DirEntry, WalkDir};

use crate::application::Application;
use crate::error::Error;
use crate::settings::Settings;
use crate::{git_pull, DevmodeError, DevmodeStatus};

use super::constants::OS_SLASH;

pub struct OpenAction {
    pub name: String,
}

impl OpenAction {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn open(&self, paths: PathBuf) -> Result<(), Error> {
        open_project(paths)
    }

    pub fn update(&self, path: PathBuf) -> Result<(), Error> {
        update_project(&self.name, path)
    }
}

pub fn open_project(path: PathBuf) -> Result<(), Error> {
    let options = Settings::current().ok_or(Error::Devmode(DevmodeError::AppSettingsNotFound))?;
    let route = path.display().to_string();
    println!("Opening {} in {}...", route, options.editor.app);
    if let Application::Custom = options.editor.app {
        let command_editor = options.editor.command;
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", format!("{command_editor} {route}").as_str()])
                .output()?;
        } else {
            run_cmd!($command_editor $route)?;
        }
    } else {
        options.editor.app.run(path.clone())?;
    }
    Ok(())
}

pub fn update_project(name: &str, path: PathBuf) -> Result<(), Error> {
    crate::report(DevmodeStatus::RepositoryUpdated(name.to_string()));
    git_pull::pull(path.as_path())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn project_paths() -> Result<Vec<PathBuf>, Error> {
    let paths: Vec<PathBuf> = WalkDir::new(home().join("Developer"))
        .max_depth(6)
        .min_depth(3)
        .contents_first(true)
        .into_iter()
        .filter_entry(|e| e.path().is_dir() && !is_hidden(e) && e.path().join(".git").exists())
        .map(|entry| entry.unwrap().path().to_path_buf())
        .collect();
    for (i, path) in paths.iter().enumerate() {
        println!("{i}: {path:?}")
    }
    Ok(paths)
}

pub fn matching_paths_for(project: &str) -> Result<Vec<PathBuf>, Error> {
    let paths: Vec<PathBuf> = project_paths()?
        .iter()
        .filter(|e| matches_project(e, project))
        .map(|entry| entry.to_path_buf())
        .collect();
    Ok(paths)
}

fn matches_project(entry: &PathBuf, path: &str) -> bool {
    entry
        .display()
        .to_string()
        .split(OS_SLASH)
        .last()
        .unwrap()
        .contains(path)
}
