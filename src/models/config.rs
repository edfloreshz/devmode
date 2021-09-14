use std::borrow::Borrow;
use std::fs;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;

use clap::App;
use serde::{Deserialize, Serialize};
use toml::de::Error;

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
    pub fn current() -> AppOptions { // TODO: return AppOptions
        let config_file = dirs::data_dir()
            .expect("Data dir not present.")
            .join("devmode/config/config.toml");
        let file = read_to_string(config_file).unwrap_or_default();
        let content = toml::from_slice(file.as_bytes())
            .unwrap_or(AppOptions::default());
        content
    }
}

impl ConfigWriter for AppOptions {
    fn write_to_config(&self) -> Result<()> {
        let data_dir = dirs::data_dir().unwrap_or(PathBuf::new()).join("devmode");
        let logs_dir = data_dir.join("logs");
        let config_dir = data_dir.join("config");
        let config_file = data_dir.join("config/config.toml");
        if !data_dir.exists() {
            fs::create_dir_all(&logs_dir)?;
            fs::create_dir_all(&config_dir)?;
            let mut file = std::fs::File::create(&config_file)?;
            file.write_all(toml::to_string(self).unwrap_or(String::new()).as_bytes())
                .expect("Unable to write data.");
            println!("Config file located at: {}", config_file.display());
        } else {
            // let file = read_to_string(&config_file)?;
            // let content: AppOptions = toml::from_str(&*file)?;
            AppOptions::current();
            // if &content == self {
            //     println!("Configuration already present.");
            // } else {
            //     std::fs::File::open(&config_file)?
            //         .write_all(toml::to_string(self).unwrap_or(String::new()).as_bytes())
            //         .expect("Unable to write data.");
            //     println!("Settings updated.")
            // }
        }
        Ok(())
    }
}
