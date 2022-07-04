use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::process::Command;

use anyhow::Result;
use anyhow::{bail, Context};
use cmd_lib::*;
use libset::config::Config;
use libset::format::FileFormat;
use libset::routes::{data, home};
use walkdir::WalkDir;

use crate::cli::select_repo;
use crate::config::application::Application;
use crate::config::settings::Settings;
use crate::constants::messages::*;

pub struct OpenAction {
    pub name: String,
}

impl OpenAction {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
    pub fn open(&self) -> Result<()> {
        let reader = create_paths_reader()?;
        let paths = find_paths(reader, &self.name)?;
        if paths.is_empty() {
            bail!(NO_PROJECT_FOUND)
        } else if paths.len() > 1 {
            eprintln!("{}", MORE_PROJECTS_FOUND); // TODO: Let user decide which
            let paths: Vec<&str> = paths.iter().map(|s| s as &str).collect();
            let path = select_repo(paths).with_context(|| "Failed to set repository.")?;
            open_project(&self.name, vec![path])?
        } else {
            open_project(&self.name, paths)?
        }
        Ok(())
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
        let mut devpaths = OpenOptions::new()
            .write(true)
            .open(data().join("devmode/devpaths"))?;
        for entry in WalkDir::new(home().join("Developer"))
            .max_depth(3)
            .min_depth(2)
        {
            let entry = entry?;
            if entry.depth() == 3 && entry.path().is_dir() {
                if let Err(e) = writeln!(devpaths, "{}", entry.path().display()) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }
        println!("Developer paths updated!");
        Ok(())
    }
}

pub fn open_project(name: &str, paths: Vec<String>) -> Result<()> {
    println!("Opening {}... \n\n{}", name, OPENING_WARNING);
    let path = &paths[0];
    let options = Config::get::<Settings>("devmode/settings.toml", FileFormat::TOML)
        .with_context(|| APP_OPTIONS_NOT_FOUND)?;
    if let Application::Custom = options.editor.app {
        let command_editor = options.editor.command;
        let route = path.replace("\\", "/").clone();
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

fn create_paths_reader() -> Result<BufReader<File>> {
    Ok(BufReader::new(File::open(data().join("devmode/devpaths"))?))
}

pub fn _get_projects() -> Result<Vec<String>> {
    let reader = create_paths_reader()?;
    let paths = reader
        .lines()
        .map(|e| e.unwrap())
        .map(|e| {
            return if e.is_empty() {
                String::new()
            } else {
                let parts = e.split('/').collect::<Vec<&str>>();
                let a = parts[0].len() + parts[1].len() + parts[2].len() + parts[3].len() + 4;
                e[a..].to_string()
            };
        })
        .collect();
    Ok(paths)
}
