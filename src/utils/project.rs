use git2::BranchType;
use git2::Repository;
// use git2::ResetType;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use anyhow::{bail, Context};
use cmd_lib::*;
use libset::routes::{data, home};
use walkdir::WalkDir;

use crate::constants::messages::*;
use crate::utils::application::Application;
use crate::utils::input::select_repo;
use crate::utils::settings::Settings;

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
    pub fn update(&self) -> Result<()> {
        let reader = create_paths_reader()?;
        let paths = find_paths(reader, &self.name)?;
        if paths.is_empty() {
            bail!(NO_PROJECT_FOUND)
        } else if paths.len() > 1 {
            eprintln!("{}", MORE_PROJECTS_FOUND); // TODO: Let user decide which
            let paths: Vec<&str> = paths.iter().map(|s| s as &str).collect();
            let path = select_repo(paths).with_context(|| "Failed to set repository.")?;
            update_project(&self.name, vec![path])?
        } else {
            update_project(&self.name, paths)?
        }
        println!("Your repository has been successfully updated!");
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
    println!("Opening {}... \n\n{}", name, OPENING_WARNING);
    let path = &paths[0];
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
    println!("Update project {}... \n\n", name);
    let path = &paths[0];
    let project = Repository::open(Path::new(path)).expect(NO_PROJECT_FOUND);
    // project.reset(&project.revparse_single("HEAD"), ResetType::Hard, None)?;

    let main_branch = if let Err(_e) = project.find_branch("main", BranchType::Local) {
        "master"
    } else {
        "main"
    };
    project
        .find_remote("origin")?
        .fetch(&[main_branch], None, None)?;
    let fetch_head = project.find_reference("FETCH_HEAD")?;
    let fetch_commit = project.reference_to_annotated_commit(&fetch_head)?;
    let analysis = project.merge_analysis(&[&fetch_commit])?;

    if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", main_branch);
        let mut reference = project.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "Fast-Forward")?;
        project.set_head(&refname)?;
        project.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    } else {
        bail!("Fast-forward only!")
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

pub fn create_paths_reader() -> Result<BufReader<File>> {
    Ok(BufReader::new(File::open(data().join("devmode/devpaths"))?))
}
