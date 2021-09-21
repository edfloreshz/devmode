use std::fmt::{Display, Formatter};

use cmd_lib::*;
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct Editor {
    pub app: EditorApp,
    pub command: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct EditorCustom {
    pub name: String,
    pub command: String,
}

impl Editor {
    pub fn new(app: EditorApp) -> Self {
        if let EditorApp::Custom(editor_custom) = app {
            Editor {
                app: EditorApp::CustomEditor,
                command: editor_custom.command,
            }
        } else {
            let command = app.command();
            Editor { app, command }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum EditorApp {
    VSCode,
    Vim,
    Custom(EditorCustom),
    CustomEditor,
    None,
}

impl Default for EditorApp {
    fn default() -> Self {
        EditorApp::None
    }
}

impl Display for EditorApp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorApp::VSCode => write!(f, "Visual Studio Code"),
            EditorApp::Vim => write!(f, "Vim"),
            _ => write!(
                f,
                "No editor set, run devmode config -e, --editor to configure it."
            ),
        }
    }
}

impl EditorApp {
    pub fn command(&self) -> String {
        String::from(match self {
            EditorApp::VSCode => "code",
            EditorApp::Vim => "vim",
            _ => "",
        })
    }
    pub fn run(&self, arg: String) -> Result<()> {
        match self {
            EditorApp::VSCode => run_cmd!(code $arg)?,
            EditorApp::Vim => run_cmd!(vim $arg)?,
            _ => {}
        }
        Ok(())
    }
    pub fn from(key: &str) -> Self {
        match key.to_lowercase().as_str() {
            "vim" => EditorApp::Vim,
            "vscode" => EditorApp::VSCode,
            "custom" => EditorApp::Custom(EditorCustom::new()),
            _ => EditorApp::None,
        }
    }
}
