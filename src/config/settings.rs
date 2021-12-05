use crate::config::editor::Editor;
use crate::constants::messages::*;
use anyhow::{Context, Result};
use libdmd::{
    config::Config,
    element::Element,
    format::{ElementFormat, FileType},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct Settings {
    pub host: String,
    pub owner: String,
    pub editor: Editor,
}

impl Settings {
    pub fn new(host: String, owner: String, editor: Editor) -> Self {
        Settings {
            host,
            owner,
            editor,
        }
    }
    pub fn init(&self) -> Result<()> {
        Config::new("devmode")
            .author("Eduardo Flores")
            .about("Development management app.")
            .version("0.1.1")
            .add(
                Element::new("config")
                    .child(Element::new("config.toml").format(ElementFormat::File)),
            )
            .add(Element::new("logs"))
            .add(Element::new("paths").child(Element::new("devpaths").format(ElementFormat::File)))
            .write()?;
        Ok(())
    }
    pub fn run(&self) -> Result<()> {
        let current_settings =
            Config::get::<Settings>("devmode/config/config.toml", FileType::TOML);
        if current_settings.is_none() {
            Config::set::<Settings>("devmode/config/config.toml", self.clone(), FileType::TOML)
                .with_context(|| FAILED_TO_WRITE_CONFIG)?;
            println!("Settings set correctly.");
        } else if self != &current_settings.with_context(|| FAILED_TO_PARSE)? {
            Config::set::<Settings>("devmode/config/config.toml", self.clone(), FileType::TOML)
                .with_context(|| FAILED_TO_WRITE_CONFIG)?;
            println!("{}", SETTINGS_UPDATED);
        } else {
            println!("{}", NO_SETTINGS_CHANGED);
        }
        Ok(())
    }
    pub fn show(&self) {
        println!(
            "Current settings: \n\
        Host: {}\n\
        Owner: {}\n\
        Editor: {}",
            self.host, self.owner, self.editor.app
        )
    }
}
