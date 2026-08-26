use dm_core::config::Config;

use crate::cli::ConfigCommand;
use crate::commands::repo::relayout;
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

            if key == "path_layout" && relayout::has_relayout_candidates()? {
                println!(
                    "note: some tracked repos no longer match this layout — run `dm repo relayout` to preview moving them"
                );
            }
        }
    }
    Ok(())
}
