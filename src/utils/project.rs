use {
    std::fs::create_dir_all,
    std::path::Path,
    cmd_lib::*,
};

use crate::Result;

pub fn open(project: &str) -> Result<()> {
    let devpath = format!(
        "{}{}",
        dirs::home_dir().unwrap().display(),
        "/.config/devmode/paths/devpaths"
    );
    let grep = run_fun!(grep $project $devpath)?;
    println!("Opening {}", project);
    run_cmd!(code $grep)?;
    Ok(())
}

pub fn make_dev_paths() -> Result<()> {
    let path = format!(
        "{}{}",
        dirs::home_dir().unwrap().display(),
        "/.config/devmode/paths"
    );
    let dev = format!("{}{}", dirs::home_dir().unwrap().display(), "/Developer");
    let devpath = format!(
        "{}{}",
        dirs::home_dir().unwrap().display(),
        "/.config/devmode/paths/devpaths"
    );
    if !Path::exists(path.as_ref()) {
        create_dir_all(&path)?
    }
    run_cmd!(find $dev -maxdepth 3 -mindepth 2 -type d -print > $devpath)?;
    Ok(())
}
