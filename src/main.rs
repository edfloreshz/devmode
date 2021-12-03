use crate::cmd::Cmd;
use anyhow::Result;
use clap::{load_yaml, App};
use colored::Colorize;

mod cli;
mod cmd;
mod config;
mod constants;

fn main() -> Result<()> {
    let yaml = load_yaml!("app.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let cmd = Cmd::new(&matches)?;
    if let Err(e) = cmd.check() {
        eprintln!("{} {}", Colorize::red("\nError:"), e);
        let error = match e.downcast_ref::<git2::Error>() {
            None => "Unknown cause.",
            Some(error) => error.message()
        };
        eprintln!("{} {}", Colorize::yellow("Caused by:"), error);
    }
    Ok(())
}
