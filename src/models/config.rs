use std::fs;
use std::fs::read_to_string;
use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::models::editor::Editor;
use crate::Result;

pub trait ConfigWriter {
    fn write_to_config(&self) -> Result<()>;
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
    pub fn current() -> AppOptions {
        let config_file = dirs::data_dir()
            .expect("Data dir not present.")
            .join("devmode/config/config.toml");
        let file = read_to_string(config_file).unwrap_or_default();
        let content = toml::from_slice(file.as_bytes()).unwrap_or_default();
        content
    }
}

impl ConfigWriter for AppOptions {
    fn write_to_config(&self) -> Result<()> {
        let data_dir = dirs::data_dir().unwrap_or_default().join("devmode");
        let logs_dir = data_dir.join("logs");
        let config_dir = data_dir.join("config");
        let config_file = data_dir.join("config/config.toml");
        if !data_dir.exists() {
            fs::create_dir_all(&logs_dir)?;
            fs::create_dir_all(&config_dir)?;
            let mut file = std::fs::File::create(&config_file)?;
            file.write_all(toml::to_string(self).unwrap_or_default().as_bytes())?;
            println!("Config file located at: {}", config_file.display());
        } else if &AppOptions::current() != self {
            std::fs::File::create(&config_file)?
                .write_all(toml::to_string(self).unwrap_or_default().as_bytes())?;
            println!("Settings updated.")
        } else {
            println!("No settings were changed.");
        }
        Ok(())
    }
}
