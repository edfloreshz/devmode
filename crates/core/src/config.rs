use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::paths;

fn default_clone_root() -> PathBuf {
    dirs_home().join("Developer")
}

fn dirs_home() -> PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub clone_root: PathBuf,
    pub default_host: String,
    pub editor: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clone_root: default_clone_root(),
            default_host: "github.com".to_string(),
            editor: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = paths::config_file()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path).map_err(|source| Error::ConfigRead {
            path: path.clone(),
            source,
        })?;
        toml::from_str(&contents).map_err(|source| Error::ConfigParse { path, source })
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::config_file()?;
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents).map_err(|source| Error::ConfigWrite { path, source })
    }

    pub fn get(&self, key: &str) -> Result<String> {
        match key {
            "clone_root" => Ok(self.clone_root.display().to_string()),
            "default_host" => Ok(self.default_host.clone()),
            "editor" => Ok(self.editor.clone().unwrap_or_default()),
            other => Err(Error::UnknownConfigKey(other.to_string())),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "clone_root" => self.clone_root = PathBuf::from(value),
            "default_host" => self.default_host = value.to_string(),
            "editor" => self.editor = Some(value.to_string()),
            other => return Err(Error::UnknownConfigKey(other.to_string())),
        }
        Ok(())
    }
}
