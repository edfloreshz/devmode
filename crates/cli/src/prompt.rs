use std::io::{IsTerminal, Write};

use dm_core::config::Config;
use dm_core::error::Error as CoreError;

use crate::error::Result;

/// Whether devmode should use `inquire`'s interactive prompts: the config
/// allows it *and* stdin is a real terminal.
///
/// The TTY check matters on Windows, where `inquire` blocks waiting on input
/// instead of erroring when there's no console, hanging scripts and CI. On
/// Unix it errors, but checking up front is cheaper and clearer.
pub fn interactive() -> bool {
    Config::load().map(|c| c.interactive).unwrap_or(false) && std::io::stdin().is_terminal()
}

/// Confirms with the user.
///
/// With `interactive` config enabled it uses `inquire::Confirm`, but errors
/// up front if there's no TTY (on Windows `inquire` would otherwise block
/// forever). With `interactive` disabled it reads one line from stdin, so
/// `yes | dm …` and similar piped input still work in scripts and CI.
pub fn confirm(prompt: &str) -> Result<bool> {
    if Config::load()?.interactive {
        if !std::io::stdin().is_terminal() {
            return Err(CoreError::from(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "can't confirm without a terminal; pass --force or set `interactive` to false",
            ))
            .into());
        }
        return Ok(inquire::Confirm::new(prompt).with_default(false).prompt()?);
    }

    print!("{prompt} [y/N] ");
    std::io::stdout().flush().map_err(CoreError::from)?;
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(CoreError::from)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}
