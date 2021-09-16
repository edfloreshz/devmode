use {cmd_lib::*, std::fs::create_dir_all, std::path::Path};

use crate::Result;

pub fn open(project: &str) -> Result<()> {
    let devpath = format!(
        "{}{}",
        dirs::data_dir().unwrap().display(),
        "/devmode/paths/devpaths"
    );
    let grep: String = run_fun!(grep $project $devpath)?;
    if grep.lines().collect::<Vec<&str>>().len() == 1 {
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
    let path = format!(
        "{}{}",
        dirs::data_dir().unwrap().display(),
        "/devmode/paths"
    );
    let dev = format!("{}{}", dirs::home_dir().unwrap().display(), "/Developer");
    let devpath = format!(
        "{}{}",
        dirs::data_dir().unwrap().display(),
        "/devmode/paths/devpaths"
    );
    if !Path::exists(path.as_ref()) {
        create_dir_all(&path)?
    }
    run_cmd!(find $dev -maxdepth 3 -mindepth 2 -type d -print > $devpath)?;
    Ok(())
}
