use dm_core::config::Config;

use crate::cli::ConfigCommand;
use crate::error::Result;

pub fn run(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Get { key } => {
            let config = Config::load()?;
            println!("{}", config.get(&key)?);
        }
        ConfigCommand::Set { key, value } => {
            let mut config = Config::load()?;
            config.set(&key, &value)?;
            config.save()?;
        }
    }
    Ok(())
}
