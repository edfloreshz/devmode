use {
    crate::cmd::cli::parse,
    clap::{load_yaml, App},
    colored::Colorize,
};

mod cmd;
mod error;
mod models;
mod utils;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    let yaml = load_yaml!("app.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let cmd = parse(&matches);
    if let Err(e) = cmd.check() {
        if let Some(e) = e.downcast_ref::<clap::Error>() {
            println!("{}", e.message)
        } else if let Some(e) = e.downcast_ref::<git2::Error>() {
            println!("{} {}", "error:".red(), e.message())
        } else if let Some(e) = e.downcast_ref::<std::io::Error>() {
            println!("{} {}", "error:".red(), e.to_string())
        } else {
            println!("{:?}", e);
        }
    }
}
