use {
    crate::cmd::cli::parse,
    anyhow::Result,
    clap::{load_yaml, App},
};

mod cmd;
mod models;
mod utils;

fn main() -> Result<()> {
    let yaml = load_yaml!("app.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let cmd = parse(&matches);
    cmd?.check()
}
