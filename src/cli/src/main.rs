use clap::StructOpt;

use cli::Cli;
use devmode::Error;

mod cli;
mod input;

fn main() -> Result<(), Error> {
    start_logger();
    let cli = Cli::parse();
    if let Err(e) = cli.run() {
        log::error!("{}", e);
    }
    Ok(())
}

pub fn start_logger() {
    std::env::set_var("RUST_LOG", "dm=info");
    pretty_env_logger::init();
}
