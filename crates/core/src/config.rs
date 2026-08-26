use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::layout::PathLayout;
use crate::paths;

fn default_clone_root() -> PathBuf {
    let path = dirs_home().join("Developer");
    path.canonicalize().unwrap_or(path)
}

fn dirs_home() -> PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub clone: CloneConfig,
    pub editor: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clone: CloneConfig::default(),
            editor: None,
        }
    }
}

/// Settings under the `[clone]` table: where repos land and how their
/// destination directory is laid out.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CloneConfig {
    pub root: PathBuf,
    pub default_host: String,
    pub path_layout: PathLayout,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            root: default_clone_root(),
            default_host: "github.com".to_string(),
            path_layout: PathLayout::default(),
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
            "clone.root" => Ok(self.clone.root.display().to_string()),
            "clone.default_host" => Ok(self.clone.default_host.clone()),
            "clone.path_layout" => Ok(self.clone.path_layout.to_config_string()),
            "editor" => Ok(self.editor.clone().unwrap_or_default()),
            other => Err(Error::UnknownConfigKey(other.to_string())),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "clone.root" => {
                let path = PathBuf::from(value);
                // Canonicalize so this matches the canonicalized paths repos
                // are tracked under — otherwise a symlinked ancestor (e.g.
                // /tmp -> /private/tmp on macOS) makes every repo look
                // mismatched even when nothing has moved. Fall back to the
                // raw path if it doesn't exist yet.
                self.clone.root = path.canonicalize().unwrap_or(path);
            }
            "clone.default_host" => self.clone.default_host = value.to_string(),
            "clone.path_layout" => self.clone.path_layout = PathLayout::parse(value)?,
            "editor" => self.editor = Some(value.to_string()),
            other => return Err(Error::UnknownConfigKey(other.to_string())),
        }
        Ok(())
    }
}
