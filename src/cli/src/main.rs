use clap::StructOpt;

use cli::Cli;
use devmode::Error;

mod cli;
mod input;

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    if let Err(e) = cli.run() {
        log::error!("{}", e);
    }
    Ok(())
}
