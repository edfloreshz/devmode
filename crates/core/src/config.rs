use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::layout::PathLayout;
use crate::paths;

fn default_clone_root() -> PathBuf {
    paths::canonicalize_best_effort(&dirs_home().join("Developer"))
}

fn dirs_home() -> PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub repo: RepoConfig,
    pub editor: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repo: RepoConfig::default(),
            editor: None,
        }
    }
}

/// Settings under the `[clone]` table: where repos land and how their
/// destination directory is laid out.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RepoConfig {
    pub root: PathBuf,
    pub host: String,
    pub layout: PathLayout,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            root: default_clone_root(),
            host: "github.com".to_string(),
            layout: PathLayout::default(),
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
            "repo.root" => Ok(self.repo.root.display().to_string()),
            "repo.host" => Ok(self.repo.host.clone()),
            "repo.layout" => Ok(self.repo.layout.to_config_string()),
            "editor" => Ok(self.editor.clone().unwrap_or_default()),
            other => Err(Error::UnknownConfigKey(other.to_string())),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "repo.root" => {
                // Canonicalize (best-effort, since the directory may not
                // exist yet) so this matches the canonicalized paths repos
                // are tracked under — otherwise a symlinked ancestor (e.g.
                // /tmp -> /private/tmp on macOS) makes every repo look
                // mismatched even when nothing has moved.
                self.repo.root = paths::canonicalize_best_effort(&PathBuf::from(value));
            }
            "repo.host" => self.repo.host = value.to_string(),
            "repo.layout" => self.repo.layout = PathLayout::parse(value)?,
            "editor" => self.editor = Some(value.to_string()),
            other => return Err(Error::UnknownConfigKey(other.to_string())),
        }
        Ok(())
    }
}
