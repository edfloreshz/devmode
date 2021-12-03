use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader};

use anyhow::Result;
use anyhow::{bail, Context};
use cmd_lib::*;
use libdmd::{data, home};
use libdmd::utils::config::Config;
use libdmd::utils::config::format::FileFormat::TOML;
use walkdir::WalkDir;
use std::io::Write;
use crate::config::editor_app::EditorApp;
use crate::config::settings::Settings;
use crate::constants::messages::*;

pub struct Project {
    pub name: Option<String>,
}

impl Project {
    pub fn open(&self) -> Result<()> {
        let reader = make_reader()?;
        let paths = find_paths(reader, self.name.as_ref().unwrap().to_string());
        if paths.is_empty() {
            bail!(NO_PROJECT_FOUND)
        } else if paths.len() > 1 {
            eprintln!("{}", MORE_PROJECTS_FOUND); // TODO: Let user decide which
            for path in paths {
                println!("{}", path)
            }
        } else {
            open_project(self.name.as_ref().unwrap().to_string(), paths)?
        }
        Ok(())
    }
    pub fn run(&self) -> Result<()> {
        if self.name.is_none() {
            bail!("Project name was not provided")
        } else {
            self.open()
        }
    }
    pub fn make_dev_paths() -> Result<()> {
        let paths_dir = data().join("devmode/paths/devpaths");
        if !paths_dir.exists() {
            create_dir_all(&paths_dir).with_context(|| "Failed to create directory.")?;
            File::create(paths_dir.join("devpaths"))?;
        }
        Project::write_paths()
    }
    fn write_paths() -> Result<()> {
        let mut devpaths = OpenOptions::new()
            .write(true)
            .open(data().join("devmode/paths/devpaths"))?;
        for entry in WalkDir::new(home().join("Developer"))
            .max_depth(3)
            .min_depth(2)
        {
            let entry = entry.unwrap();
            if entry.depth() == 3 && entry.path().is_dir() {
                if let Err(e) = writeln!(devpaths, "{}", entry.path().display().to_string()) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }
        Ok(())
    }
}

pub fn open_project(name: String, paths: Vec<String>) -> Result<()> {
    println!("Opening {}... \n\n {}", name, OPENING_WARNING);
    let path = &paths[0];
    let options = Config::get::<Settings>("devmode/config/config.toml", TOML)
        .with_context(|| APP_OPTIONS_NOT_FOUND)?;
    if let EditorApp::Custom = options.editor.app {
        let command_editor = options.editor.command;
        let route = path.clone();
        run_cmd!($command_editor $route)?;
    } else {
        options.editor.app.run(path.clone())?;
    }
    Ok(())
}

pub fn find_paths(reader: BufReader<File>, path: String) -> Vec<String> {
    reader
        .lines()
        .map(|e| e.unwrap())
        .filter(|e| {
            let split: Vec<&str> = e.split('/').collect();
            split.last().unwrap() == &path
        })
        .collect::<Vec<String>>()
}

fn make_reader() -> Result<BufReader<File>> {
    todo!()
    // Ok(BufReader::new(File::open(data().join(DEVPATHS_FILE))?))
}

pub fn _get_projects() -> Result<Vec<String>> {
    let reader = make_reader()?;
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
