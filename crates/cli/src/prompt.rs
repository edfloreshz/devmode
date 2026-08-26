use std::io::Write;

use dm_core::config::Config;
use dm_core::error::Error as CoreError;

use crate::error::Result;

/// Confirms with the user. When `interactive` config is disabled, falls back
/// to a plain stdin read instead of `inquire::Confirm`, which hard-errors
/// without a real TTY — this is what lets `dm` be piped in scripts/CI.
pub fn confirm(prompt: &str) -> Result<bool> {
    if Config::load()?.interactive {
        Ok(inquire::Confirm::new(prompt).with_default(false).prompt()?)
    } else {
        print!("{prompt} [y/N] ");
        std::io::stdout().flush().map_err(CoreError::from)?;
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(CoreError::from)?;
        Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
    }
}
