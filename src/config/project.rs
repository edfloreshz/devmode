use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader};

use crate::config::editor_app::EditorApp;
use crate::config::settings::Settings;
use crate::constants::messages::*;
use anyhow::Result;
use anyhow::{bail, Context};
use cmd_lib::*;
use libdmd::config::Config;
use libdmd::format::FileType;
use libdmd::routes::{data, home};
use std::io::Write;
use walkdir::WalkDir;

pub struct Project {
    pub name: String,
}

impl Project {
    pub fn new(name: &String) -> Self {
        Self { name: name.clone() }
    }
    pub fn open(&self) -> Result<()> {
        let reader = make_reader()?;
        let paths = find_paths(reader, &self.name)?;
        if paths.is_empty() {
            bail!(NO_PROJECT_FOUND)
        } else if paths.len() > 1 {
            eprintln!("{}", MORE_PROJECTS_FOUND); // TODO: Let user decide which
            for path in paths {
                println!("{}", path)
            }
        } else {
            open_project(&self.name, paths)?
        }
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        self.open()
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
            let entry = entry?;
            if entry.depth() == 3 && entry.path().is_dir() {
                if let Err(e) = writeln!(devpaths, "{}", entry.path().display().to_string()) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }
        Ok(())
    }
}

pub fn open_project(name: &String, paths: Vec<String>) -> Result<()> {
    println!("Opening {}... \n\n {}", name, OPENING_WARNING);
    let path = &paths[0];
    let options = Config::get::<Settings>("devmode/config/config.toml", FileType::TOML)
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

pub fn find_paths(reader: BufReader<File>, path: &String) -> Result<Vec<String>> {
    let paths = reader
        .lines()
        .map(|e| e.unwrap_or_default())
        .filter(|e| {
            let split: Vec<&str> = e.split('/').collect();
            split.last().unwrap() == &path
        })
        .collect::<Vec<String>>();
    Ok(paths)
}

fn make_reader() -> Result<BufReader<File>> {
    Ok(BufReader::new(File::open(
        data().join("devmode/paths/devpaths"),
    )?))
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
