use crate::constants::constants::commands::*;
use crate::constants::constants::messages::NO_EDITOR_SET;
use crate::constants::constants::names::*;
use anyhow::Result;
use cmd_lib::run_cmd;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum EditorApp {
    VSCode,
    Vim,
    Custom,
    None,
}

impl EditorApp {
    pub fn command(&self) -> String {
        String::from(match self {
            EditorApp::VSCode => VSCODE_CMD,
            EditorApp::Vim => VIM_CMD,
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
            _ => EditorApp::None,
        }
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        EditorApp::None
    }
}

impl Display for EditorApp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorApp::VSCode => write!(f, "{}", VSCODE_NAME),
            EditorApp::Vim => write!(f, "{}", VIM_NAME),
            EditorApp::Custom => write!(f, "{}", CUSTOM_NAME),
            _ => write!(f, "{}", NO_EDITOR_SET),
        }
    }
}
