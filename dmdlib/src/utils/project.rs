use std::fs::create_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{BufRead, BufReader};

use anyhow::Result;
use anyhow::{bail, Context};
use cmd_lib::*;
use walkdir::WalkDir;

use crate::utils::constants::messages::*;
use crate::utils::constants::paths::files::DEVPATHS_FILE;
use crate::utils::constants::paths::folders::{DEVELOPER_DIR, PATHS_DIR};
use crate::utils::{config::AppOptions, editor::EditorApp};
use crate::{data, home};

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

    pub fn make_dev_paths() -> Result<()> {
        let paths_dir = data().join(PATHS_DIR);
        if !paths_dir.exists() {
            create_dir_all(paths_dir).with_context(|| failed_to("crate", "paths directory"))?;
            File::create(data().join(DEVPATHS_FILE))?;
        }
        Project::write_paths()
    }
    fn write_paths() -> Result<()> {
        let mut devpaths = OpenOptions::new()
            .write(true)
            .open(data().join(DEVPATHS_FILE))?;
        for entry in WalkDir::new(home().join(DEVELOPER_DIR))
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
    let options = AppOptions::current().unwrap();
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
    Ok(BufReader::new(File::open(data().join(DEVPATHS_FILE))?))
}

pub fn get_projects() -> Result<Vec<String>> {
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
