use std::fs::create_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::Context;
use walkdir::WalkDir;

use crate::models::config::AppOptions;
use crate::Result;

pub struct Project<'a> {
    pub name: Option<&'a str>,
}

impl<'a> Project<'a> {
    pub fn open(&self) -> Result<()> {
        let devpath = dirs::data_dir().unwrap().join("devmode/paths/devpaths");
        let reader = BufReader::new(File::open(devpath)?);
        let paths = reader
            .lines()
            .map(|e| e.unwrap())
            .filter(|e| {
                let split: Vec<&str> = e.split('/').collect();
                split.last().unwrap() == &self.name.unwrap()
            })
            .collect::<Vec<String>>();
        if paths.is_empty() {
            println!(
                "No project was found.\n\
        If you know this project exists, run `devmode config -m, --map` to refresh the paths file."
            );
        } else if paths.len() > 1 {
            println!("Two or more projects found."); // TODO: Let user decide which
            for path in paths {
                println!("{}", path)
            }
        } else {
            println!("Opening {}", self.name.unwrap());
            let path = &paths[0];
            AppOptions::current()
                .unwrap()
                .editor
                .app
                .run(path.clone())?
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
