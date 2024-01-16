use anyhow::{Context, Result};
use colored::Colorize;
use libset::element::Content;
use libset::{config::Config, format::FileFormat, new_file};
use serde::{Deserialize, Serialize};

use crate::commands::editor::Editor;
use crate::constants::messages::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct Settings {
    pub host: String,
    pub owner: String,
    pub editor: Editor,
    pub workspaces: Workspaces,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct Workspaces {
    pub(crate) names: Vec<String>,
}

impl Content for Settings {}

impl Settings {
    pub fn new(host: String, owner: String, editor: Editor) -> Self {
        Settings {
            host,
            owner,
            editor,
            workspaces: Workspaces { names: vec![] },
        }
    }
    pub fn init() -> Result<()> {
        Config::new("devmode")
            .author("Eduardo Flores")
            .about("Development management app.")
            .version("0.1.1")
            .add(
                new_file!("settings.toml")
                    .set_format(FileFormat::TOML)
                    .set_content(Box::new(Settings::default())),
            )
            .add(new_file!("devpaths"))
            .write()?;
        Ok(())
    }
    pub fn write(&self, hide_output: bool) -> Result<()> {
        let current_settings = Settings::current();
        if current_settings.is_none() {
            Config::set::<Settings>("devmode/settings.toml", self.clone(), FileFormat::TOML)
                .with_context(|| FAILED_TO_WRITE_CONFIG)?;
            println!("Settings set correctly.");
        } else if self != &current_settings.with_context(|| FAILED_TO_PARSE)? {
            Config::set::<Settings>("devmode/settings.toml", self.clone(), FileFormat::TOML)
                .with_context(|| FAILED_TO_WRITE_CONFIG)?;
            if !hide_output {
                println!("{}", SETTINGS_UPDATED);
            }
        } else if !hide_output {
            println!("{}", NO_SETTINGS_CHANGED);
        }
        Ok(())
    }
    pub fn show(&self) {
        println!(
            "{}\n{}{}\n{}{}\n{}{}\n{}{:?}",
            Colorize::yellow("Current settings:"),
            Colorize::green("Host: "),
            self.host,
            Colorize::red("Owner: "),
            self.owner,
            Colorize::blue("Editor: "),
            self.editor.app,
            Colorize::purple("Workspaces: "),
            self.workspaces.names
        );
    }
    pub fn current() -> Option<Settings> {
        Config::get::<Settings>("devmode/settings.toml", FileFormat::TOML)
    }
}
