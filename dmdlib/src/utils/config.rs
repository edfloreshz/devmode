use std::fs;
use std::fs::read_to_string;
use std::io::Write;

use anyhow::Context;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::data;
use crate::utils::constants::messages::*;
use crate::utils::constants::paths::files::*;
use crate::utils::constants::paths::folders::*;
use crate::utils::editor::Editor;

pub trait ConfigWriter {
    fn write_to_config(&self) -> Result<()>;
    fn write(&self) -> Result<()>;
    fn initialize(&self) -> Result<()>;
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct AppOptions {
    pub host: String,
    pub owner: String,
    pub editor: Editor,
}

impl AppOptions {
    pub fn new(host: String, owner: String, editor: Editor) -> Self {
        AppOptions {
            host,
            owner,
            editor,
        }
    }
    pub fn current() -> Option<AppOptions> {
        let config_file = data().join(CONFIG_FILE);
        Option::from(if !config_file.exists() {
            AppOptions::default()
        } else {
            let file = read_to_string(config_file).unwrap();
            let content = toml::from_slice(file.as_bytes()).unwrap_or_default();
            content
        })
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

impl ConfigWriter for AppOptions {
    fn write_to_config(&self) -> Result<()> {
        let data_dir = data().join(DEVMODE_DIR);
        if !data_dir.exists() {
            self.initialize()?;
            self.write()
        } else if &AppOptions::current().unwrap() != self {
            self.write()
        } else {
            println!("{}", NO_SETTINGS_CHANGED);
            Ok(())
        }
    }

    fn write(&self) -> Result<()> {
        std::fs::File::create(data().join(CONFIG_FILE))
            .with_context(|| failed_to("open", "config.toml"))?
            .write_all(
                toml::to_string(self)
                    .with_context(|| FAILED_TO_PARSE)?
                    .as_bytes(),
            )
            .with_context(|| FAILED_TO_WRITE_CONFIG)?;
        println!("{}", SETTINGS_UPDATED);
        Ok(())
    }

    fn initialize(&self) -> Result<()> {
        let config = data().join(CONFIG_DIR);
        fs::create_dir_all(data().join(LOGS_DIR))
            .with_context(|| failed_to("create", "logs directory"))?;
        fs::create_dir_all(&config).with_context(|| failed_to("create", "config directory"))?;
        println!("Config file located at: {}", config.display());
        Ok(())
    }
}
