use ::devmode::Error;
use clap::Parser;
use cli::Cli;

pub mod cli;

fn main() -> Result<(), Error> {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_LOG", "trace");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::init();
    let cli = Cli::parse();
    if let Err(e) = cli.run() {
        log::error!("{e}")
    }
    Ok(())
}
