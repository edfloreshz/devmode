use {
    anyhow::Result,
    clap::{App, load_yaml},
    crate::models::cmd::Cmd,
};

mod cmd;
mod models;

fn main() -> Result<()> {
    let yaml = load_yaml!("app.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let cmd = Cmd::new(&matches);
    cmd?.check()
}
