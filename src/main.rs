use {
    crate::cmd::cli::parse,
    clap::{load_yaml, App},
    crate::error::custom::Error,
};
use crate::error::custom::downcast_err;

mod cmd;
mod error;
mod models;
mod utils;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    let yaml = load_yaml!("app.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let cmd = parse(&matches);
    let check = cmd.check();
    downcast_err(check)
}
