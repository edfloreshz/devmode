use {
    crate::Result, anyhow::Context, cmd_lib::*, std::fs::create_dir_all, std::fs::OpenOptions,
    std::io::Write, std::path::Path, walkdir::WalkDir,
};

pub fn open(project: &str) -> Result<()> {
    let devpath = format!(
        "{}{}",
        dirs::data_dir().unwrap().display(),
        "/devmode/paths/devpaths"
    );
    let grep: String = run_fun!(grep $project $devpath)?; //TODO: Manage errors
    if grep.lines().count() == 1 {
        println!("Opening {}", project);
        run_cmd!(code $grep)?;
    }
    println!("Two or more projects found: \n");
    for line in grep.lines() {
        println!("-> {}", line)
        // TODO: Let the user decide.
    }
    Ok(())
}

pub fn make_dev_paths() -> Result<()> {
    let paths_dir = dirs::data_dir().unwrap().join("devmode/paths");
    println!("{}", paths_dir.display());
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
