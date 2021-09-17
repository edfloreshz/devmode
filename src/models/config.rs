use anyhow::Context;
use {
    crate::models::editor::Editor,
    crate::Result,
    serde::{Deserialize, Serialize},
    std::fs,
    std::fs::read_to_string,
    std::io::Write,
};

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
    pub fn current() -> Option<AppOptions> {
        let config_file = dirs::data_dir()
            .expect("Data dir not present.")
            .join("devmode/config/config.toml");
        if !config_file.exists() {
            None
        } else {
            let file = read_to_string(config_file).unwrap();
            let content = toml::from_slice(file.as_bytes()).unwrap_or_default();
            Some(content)
        }
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
        let data_dir = dirs::data_dir().unwrap_or_default().join("devmode");
        let logs_dir = data_dir.join("logs");
        let config_dir = data_dir.join("config");
        let config_file = data_dir.join("config/config.toml");
        if !data_dir.exists() {
            fs::create_dir_all(&logs_dir).with_context(|| {
                "Failed to create the `logs` directory, you may need additional permissions."
            })?;
            fs::create_dir_all(&config_dir).with_context(|| {
                "Failed to create the `config` directory, you may need additional permissions."
            })?;
            let mut file = std::fs::File::create(&config_file).with_context(|| {
                "Failed to create `config.toml`, you may need additional permissions."
            })?;
            file.write_all(
                toml::to_string(self)
                    .with_context(|| "Failed to parse app options.")?
                    .as_bytes(),
            )
            .with_context(|| "Failed to write changes to `config.toml`.")?;
            println!("Config file located at: {}", config_file.display());
        } else if &AppOptions::current().unwrap() != self {
            std::fs::File::create(&config_file)
                .with_context(|| {
                    "Failed to open `config.toml`, you may need additional permissions."
                })?
                .write_all(
                    toml::to_string(self)
                        .with_context(|| "Failed to parse app options.")?
                        .as_bytes(),
                )
                .with_context(|| "Failed to write changes to `config.toml`.")?;
            println!("Settings updated.")
        } else {
            println!("No settings were changed.");
        }
        Ok(())
    }
}
