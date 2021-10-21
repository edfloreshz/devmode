use std::fs::create_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Context};
use anyhow::Result;
use cmd_lib::*;
use walkdir::WalkDir;

use crate::models::{config::AppOptions, editor::EditorApp};

pub struct Project<'a> {
    pub name: Option<&'a str>,
}

impl<'a> Project<'a> {
    pub fn open(&self) -> Result<()> {
        let reader = BufReader::new(File::open(
            dirs::data_dir().unwrap().join("devmode/paths/devpaths"),
        )?);
        let paths = reader
            .lines()
            .map(|e| e.unwrap())
            .filter(|e| {
                let split: Vec<&str> = e.split('/').collect();
                split.last().unwrap() == &self.name.unwrap()
            })
            .collect::<Vec<String>>();
        if paths.is_empty() {
            bail!(
                "No project was found.\n\
        If you know this project exists, run `devmode config -m, --map` to refresh the paths file."
            )
        } else if paths.len() > 1 {
            eprintln!("Two or more projects found."); // TODO: Let user decide which
            for path in paths {
                println!("{}", path)
            }
        } else {
            println!(
                "Opening {}... \n\n\
            If the editor does not support openning from a path, you'll have to open it yourself.",
                self.name.unwrap()
            );
            let path = &paths[0];
            let options = AppOptions::current().unwrap();
            if let EditorApp::Custom = options.editor.app {
                let command_editor = options.editor.command;
                let route = path.clone();
                run_cmd!($command_editor $route)?
            } else {
                options.editor.app.run(path.clone())?
            }
        }
        Ok(())
    }
    pub fn make_dev_paths() -> Result<()> {
        let paths_dir = dirs::data_dir().unwrap().join("devmode/paths");
        let paths_file = paths_dir.join("devpaths");
        if !Path::exists(paths_dir.as_path()) {
            create_dir_all(paths_dir.as_path())
                .with_context(|| "Failed to create `paths` directory.")?;
            File::create(paths_file.clone())?;
        }
        let mut devpaths = OpenOptions::new().write(true).open(paths_file)?;
        for entry in WalkDir::new(dirs::home_dir().unwrap().join("Developer"))
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

pub fn get_projects() -> Result<Vec<String>> {
    let reader = BufReader::new(File::open(
        dirs::data_dir().unwrap().join("devmode/paths/devpaths"),
    )?);
    let paths = reader
        .lines()
        .map(|e| e.unwrap())
        .map(|e| {
            let parts = e.split("/").collect::<Vec<&str>>();
            let a = &parts[0].len() + &parts[1].len() + &parts[2].len() + &parts[3].len() + 4;
            e[a..].to_string()
        })
        .collect();
    Ok(paths)
}
