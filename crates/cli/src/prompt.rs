use crate::error::Result;

pub fn confirm(prompt: &str) -> Result<bool> {
    Ok(inquire::Confirm::new(prompt).with_default(false).prompt()?)
}
