use anyhow::{Context, Result};
use colored::Colorize;
use libset::{
    config::Config,
    element::{Element, ElementType},
    format::FileFormat,
};
use serde::{Deserialize, Serialize};

use crate::config::editor::Editor;
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
    pub fn init() -> Result<()> {
        Config::new("devmode")
            .author("Eduardo Flores")
            .about("Development management app.")
            .version("0.1.1")
            .add(
                Element::new("config")
                    .add_child(Element::new("config.toml").set_type(ElementType::File)),
            )
            .add(Element::new("logs"))
            .add(
                Element::new("paths")
                    .add_child(Element::new("devpaths").set_type(ElementType::File)),
            )
            .write()?;
        Ok(())
    }
    pub fn write(&self) -> Result<()> {
        println!();
        let current_settings =
            Config::get::<Settings>("devmode/config/config.toml", FileFormat::TOML);
        if current_settings.is_none() {
            Config::set::<Settings>("devmode/config/config.toml", self.clone(), FileFormat::TOML)
                .with_context(|| FAILED_TO_WRITE_CONFIG)?;
            println!("Settings set correctly.");
        } else if self != &current_settings.with_context(|| FAILED_TO_PARSE)? {
            Config::set::<Settings>("devmode/config/config.toml", self.clone(), FileFormat::TOML)
                .with_context(|| FAILED_TO_WRITE_CONFIG)?;
            println!("{}", SETTINGS_UPDATED);
        } else {
            println!("{}", NO_SETTINGS_CHANGED);
        }
        println!();
        Ok(())
    }
    pub fn show(&self) {
        println!(
            "{}\n{}{}\n{}{}\n{}{}",
            Colorize::yellow("Current settings:"),
            Colorize::green("Host: "),
            self.host,
            Colorize::red("Owner: "),
            self.owner,
            Colorize::blue("Editor: "),
            self.editor.app
        );
    }
}
