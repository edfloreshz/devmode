use std::io::Write;

use dm_core::error::Error as CoreError;

use crate::error::Result;

pub fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    std::io::stdout().flush().map_err(CoreError::from)?;
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(CoreError::from)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}
