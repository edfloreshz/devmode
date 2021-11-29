use crate::cmd::Cmd;
use anyhow::Result;
use clap::{load_yaml, App};

mod cli;
mod cmd;
mod config;
mod constants;

fn main() -> Result<()> {
    let yaml = load_yaml!("app.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let cmd = Cmd::new(&matches)?;
    cmd.check()
}
