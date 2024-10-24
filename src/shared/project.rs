use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;

use cmd_lib::*;
use libset::routes::{data, home};
use walkdir::WalkDir;

use crate::application::Application;
use crate::error::Error;
use crate::settings::Settings;
use crate::{git_pull, DevmodeError, DevmodeStatus};

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

    pub fn make_dev_paths() -> Result<(), Error> {
        let paths_dir = data().join("devmode/devpaths");
        if !paths_dir.exists() {
            create_dir_all(&paths_dir)?;
            File::create(paths_dir.join("devpaths"))?;
        }
        OpenAction::write_paths()
    }

    fn write_paths() -> Result<(), Error> {
        let settings =
            Settings::current().ok_or(Error::Devmode(DevmodeError::AppSettingsNotFound))?;
        let mut devpaths = OpenOptions::new()
            .write(true)
            .open(data().join("devmode/devpaths"))?;
        for entry in WalkDir::new(home().join("Developer"))
            .max_depth(4)
            .min_depth(2)
        {
            let entry = entry?;
            let repo = entry.path().to_str().unwrap().to_string();
            let repo = repo
                .split(if cfg!(target_os = "windows") {
                    "\\"
                } else {
                    "/"
                })
                .last()
                .unwrap();
            let parent = entry.path().parent().unwrap().to_str().unwrap().to_string();
            let workspace = parent
                .split(if cfg!(target_os = "windows") {
                    "\\"
                } else {
                    "/"
                })
                .last()
                .unwrap();
            if (entry.depth().eq(&3) && !settings.workspaces.names.contains(&repo.to_string()))
                || (entry.depth().eq(&4)
                    && settings.workspaces.names.contains(&workspace.to_string()))
                    && entry.path().is_dir()
            {
                if let Err(e) = writeln!(devpaths, "{}", entry.path().display()) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }
        crate::report(DevmodeStatus::RepositoryCloned);
        Ok(())
    }
}

pub fn open_project(path: PathBuf) -> Result<(), Error> {
    let options = Settings::current().ok_or(Error::Devmode(DevmodeError::AppSettingsNotFound))?;
    let route = path.display().to_string();
    println!("Opening {} in {}...", route, options.editor.app.to_string(),);
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

pub fn find_paths(path: &str) -> Result<Vec<PathBuf>, Error> {
    let reader = BufReader::new(File::open(data().join("devmode/devpaths"))?);
    let paths = reader
        .lines()
        .filter_map(|e| {
            if let Ok(line) = e {
                let split: Vec<&str> = line
                    .split(if cfg!(target_os = "windows") {
                        "\\"
                    } else {
                        "/"
                    })
                    .collect();
                if split.last().unwrap().eq(&path) {
                    return Some(PathBuf::from(line));
                }
            }
            None
        })
        .collect::<Vec<PathBuf>>();
    Ok(paths)
}
