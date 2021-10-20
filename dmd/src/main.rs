use anyhow::Result;
use clap::{App, load_yaml};
use crate::cmd::Cmd;

mod cmd;
mod cli;

fn main() -> Result<()> {
    let yaml = load_yaml!("app.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let cmd = Cmd::new(&matches)?;
    cmd.check()
}