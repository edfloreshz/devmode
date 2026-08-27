use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::layout::PathLayout;
use crate::paths;

fn default_clone_root() -> PathBuf {
    paths::normalize_path(&dirs_home().join("Developer"))
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
    /// When `false`, the CLI never uses TTY-requiring interactive prompts
    /// (`inquire::Confirm`/`Select`), confirmations fall back to a plain
    /// stdin read and ambiguous repo lookups error instead of offering a
    /// picker, so devmode stays usable from scripts and pipes.
    pub interactive: bool,
    pub ui: UiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repo: RepoConfig::default(),
            editor: None,
            interactive: true,
            ui: UiConfig::default(),
        }
    }
}

/// Settings under the `[ui]` table, used by the desktop app. The CLI and TUI
/// ignore these; they live in the same file so all three frontends keep one
/// config, and so `dm config` can read and write them like anything else.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub theme_mode: ThemeMode,
    /// Theme names, applied according to `theme_mode`. Which names are valid
    /// is the GUI's business, it owns the theme list, so they're plain
    /// strings here, and an unrecognised one falls back to a built-in.
    pub light_theme: String,
    pub dark_theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::default(),
            light_theme: "Light".to_string(),
            dark_theme: "Dark".to_string(),
        }
    }
}

/// Which of the two configured themes the app uses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    /// Follow the desktop's light/dark preference, live.
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark];

    pub fn as_str(self) -> &'static str {
        match self {
            ThemeMode::System => "system",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "system" => Ok(ThemeMode::System),
            "light" => Ok(ThemeMode::Light),
            "dark" => Ok(ThemeMode::Dark),
            other => Err(Error::InvalidConfigValue {
                key: "ui.theme_mode".to_string(),
                value: other.to_string(),
                expected: "system, light, or dark".to_string(),
            }),
        }
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Title case reads better in the GUI's picker, which renders this.
        f.write_str(match self {
            ThemeMode::System => "Follow system",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        })
    }
}

/// Settings under the `[clone]` table: where repos land and how their
/// destination directory is laid out.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RepoConfig {
    pub root: PathBuf,
    pub layout: PathLayout,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            root: default_clone_root(),
            layout: PathLayout::default(),
        }
    }
}

impl Config {
    pub fn is_saved() -> bool {
        paths::config_file()
            .map(|path| path.exists())
            .unwrap_or(false)
    }

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
            "repo.layout" => Ok(self.repo.layout.to_config_string()),
            "editor" => Ok(self.editor.clone().unwrap_or_default()),
            "interactive" => Ok(self.interactive.to_string()),
            "ui.theme_mode" => Ok(self.ui.theme_mode.as_str().to_string()),
            "ui.light_theme" => Ok(self.ui.light_theme.clone()),
            "ui.dark_theme" => Ok(self.ui.dark_theme.clone()),
            other => Err(Error::UnknownConfigKey(other.to_string())),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            // Normalize (not canonicalize, see paths::normalize_path) so
            // this matches how repo paths are stored, regardless of whether
            // the directory exists yet or any ancestor is a symlink.
            "repo.root" => self.repo.root = paths::normalize_path(&PathBuf::from(value)),
            "repo.layout" => self.repo.layout = PathLayout::parse(value)?,
            "editor" => self.editor = Some(value.to_string()),
            "interactive" => {
                self.interactive = value
                    .parse::<bool>()
                    .map_err(|_| Error::InvalidConfigValue {
                        key: key.to_string(),
                        value: value.to_string(),
                        expected: "true or false".to_string(),
                    })?
            }
            "ui.theme_mode" => self.ui.theme_mode = ThemeMode::parse(value)?,
            "ui.light_theme" => self.ui.light_theme = value.to_string(),
            "ui.dark_theme" => self.ui.dark_theme = value.to_string(),
            other => return Err(Error::UnknownConfigKey(other.to_string())),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_config_without_a_ui_table_still_parses() {
        // Files written before `[ui]` existed must keep loading.
        let config: Config = toml::from_str(
            r#"
            interactive = true

            [repo]
            root = "/code"
            host = "github.com"

            [repo.layout]
            kind = "flat"
            "#,
        )
        .expect("an older config should still parse");

        assert_eq!(config.ui.theme_mode, ThemeMode::System);
        assert_eq!(config.ui.light_theme, "Light");
        assert_eq!(config.ui.dark_theme, "Dark");
    }

    #[test]
    fn ui_keys_round_trip_through_get_and_set() {
        let mut config = Config::default();

        config.set("ui.theme_mode", "dark").unwrap();
        config.set("ui.light_theme", "Solarized Light").unwrap();
        config.set("ui.dark_theme", "Tokyo Night").unwrap();

        assert_eq!(config.get("ui.theme_mode").unwrap(), "dark");
        assert_eq!(config.get("ui.light_theme").unwrap(), "Solarized Light");
        assert_eq!(config.get("ui.dark_theme").unwrap(), "Tokyo Night");

        let reparsed: Config = toml::from_str(&toml::to_string_pretty(&config).unwrap()).unwrap();
        assert_eq!(reparsed.ui.theme_mode, ThemeMode::Dark);
        assert_eq!(reparsed.ui.dark_theme, "Tokyo Night");
    }

    #[test]
    fn an_unknown_theme_mode_is_rejected() {
        let mut config = Config::default();

        assert!(config.set("ui.theme_mode", "sepia").is_err());
        assert_eq!(config.ui.theme_mode, ThemeMode::System);
    }
}
