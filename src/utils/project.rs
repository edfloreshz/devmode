use crate::models::config::AppOptions;
use std::fs::File;
use std::io::{BufRead, BufReader};
use {
    crate::Result, anyhow::Context, std::fs::create_dir_all, std::fs::OpenOptions, std::io::Write,
    std::path::Path, walkdir::WalkDir,
};

pub fn open(project: &str) -> Result<()> {
    let devpath = dirs::data_dir().unwrap().join("devmode/paths/devpaths");
    let reader = BufReader::new(File::open(devpath)?);
    let paths = reader
        .lines()
        .map(|e| e.unwrap())
        .filter(|e| {
            let split: Vec<&str> = e.split('/').collect();
            split.last().unwrap() == &project
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
        println!("Opening {}", project);
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
    let mut devpaths = OpenOptions::new()
        .write(true)
        .open(dirs::data_dir().unwrap().join("devmode/paths/devpaths"))?;
    if !Path::exists(paths_dir.as_path()) {
        create_dir_all(paths_dir.as_path())
            .with_context(|| "Failed to create `paths` directory.")?;
    }
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
