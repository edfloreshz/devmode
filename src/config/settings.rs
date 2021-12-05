use crate::config::editor::Editor;
use anyhow::{Context, Result};
use libdmd::utils::config::{Config, Element, Format};
use libdmd::utils::config::format::FileFormat::TOML;
use serde::{Deserialize, Serialize};
use crate::constants::messages::*;

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
    pub fn init(&self) -> Result<Config> {
        Config::new()
            .name("devmode")
            .author("Eduardo Flores")
            .about("Development management app.")
            .version("0.1.1")
            .add(Element::new("config").child(Element::new("config.toml").format(Format::File)))
            .add(Element::new("logs"))
            .add(Element::new("paths").child(Element::new("devpaths").format(Format::File)))
            .write()
    }
    pub fn run(&self) -> Result<()> {
        let current_settings = Config::get::<Settings>("devmode/config/config.toml", TOML);
        if current_settings.is_none() {
            Config::set::<Settings>("devmode/config/config.toml", self.clone(), TOML).with_context(|| FAILED_TO_WRITE_CONFIG)?;
            println!("Settings set correctly.");
        } else if self != &current_settings.with_context(|| FAILED_TO_PARSE)? {
            Config::set::<Settings>("devmode/config/config.toml", self.clone(), TOML).with_context(|| FAILED_TO_WRITE_CONFIG)?;
            println!("{}", SETTINGS_UPDATED);
        }
        else {
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
