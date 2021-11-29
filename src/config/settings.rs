use crate::config::editor::Editor;
use anyhow::Result;
use libdmd::utils::config::config::{Config, Element, Format};
use serde::{Deserialize, Serialize};

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
    pub fn init(&self) -> Result<Config> {
        Config::new()
            .name("devmode")
            .author("Eduardo Flores")
            .about("Development management app.")
            .version("0.1.1")
            .add(Element::new("config").child(Element::new("config.toml").format(Format::File)))
            .add(Element::new("logs"))
            .add(Element::new("paths").child(Element::new("devpaths").format(Format::File)))
            .write()
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
