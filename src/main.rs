use {
    crate::models::cmd::Cmd,
    anyhow::Result,
    clap::{load_yaml, App},
};

mod cmd;
mod models;

fn main() -> Result<()> {
    let yaml = load_yaml!("app.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let cmd = Cmd::new(&matches);
    cmd?.check()
}
