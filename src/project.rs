use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use cmd_lib::*;
use libset::routes::{data, home};
use walkdir::WalkDir;

use crate::application::Application;
use crate::constants::messages::*;
use crate::git_pull;
use crate::settings::Settings;

pub struct OpenAction {
    pub name: String,
}

impl OpenAction {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn open(&self, paths: Vec<String>) -> Result<()> {
        open_project(&self.name, paths)
    }

    pub fn update(&self, paths: Vec<String>) -> Result<()> {
        update_project(&self.name, paths)
    }

    pub fn make_dev_paths() -> Result<()> {
        let paths_dir = data().join("devmode/devpaths");
        if !paths_dir.exists() {
            create_dir_all(&paths_dir).with_context(|| "Failed to create directory.")?;
            File::create(paths_dir.join("devpaths"))?;
        }
        OpenAction::write_paths()
    }
    fn write_paths() -> Result<()> {
        let settings = Settings::current().with_context(|| "Failed to get settings.")?;
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
            if (entry.depth() == 3 && !settings.workspaces.names.contains(&repo.to_string()))
                || (entry.depth() == 4
                    && settings.workspaces.names.contains(&workspace.to_string()))
                    && entry.path().is_dir()
            {
                if let Err(e) = writeln!(devpaths, "{}", entry.path().display()) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }
        println!("Repository cloned successfully! 🎉️");
        Ok(())
    }
}

pub fn open_project(name: &str, paths: Vec<String>) -> Result<()> {
    let path = &paths[0];
    println!(
        "Opening {} in {}... \n\n{}",
        name,
        path.clone(),
        OPENING_WARNING
    );
    git_pull::status_short(path.clone())?;
    let options = Settings::current().with_context(|| APP_OPTIONS_NOT_FOUND)?;
    if let Application::Custom = options.editor.app {
        let command_editor = options.editor.command;
        let route = path.replace('\\', "/");
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

pub fn update_project(name: &str, paths: Vec<String>) -> Result<()> {
    println!("Updating project {}... \n\n", name);
    let path = &paths[0];

    git_pull::pull(Path::new(path))
}

pub fn find_paths(reader: BufReader<File>, path: &str) -> Result<Vec<String>> {
    let paths = reader
        .lines()
        .map(|e| e.unwrap_or_default())
        .filter(|e| {
            let split: Vec<&str> = e
                .split(if cfg!(target_os = "windows") {
                    "\\"
                } else {
                    "/"
                })
                .collect();
            split.last().unwrap() == &path
        })
        .collect::<Vec<String>>();
    Ok(paths)
}

pub fn create_paths_reader() -> Result<BufReader<File>> {
    Ok(BufReader::new(File::open(data().join("devmode/devpaths"))?))
}
