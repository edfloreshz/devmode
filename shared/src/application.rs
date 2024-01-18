use std::fmt::{Display, Formatter};
use std::process::Command;

use anyhow::Result;
use cmd_lib::run_cmd;
use serde::{Deserialize, Serialize};

use crate::constants::commands::*;
use crate::constants::messages::NO_EDITOR_SET;
use crate::constants::names::*;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Application {
    VSCode,
    Vim,
    Custom,
    None,
}

impl Application {
    pub fn command(&self) -> String {
        String::from(match self {
            Application::VSCode => VSCODE_CMD,
            Application::Vim => VIM_CMD,
            _ => "",
        })
    }
    pub fn run(&self, arg: String) -> Result<()> {
        match self {
            Application::VSCode => {
                if cfg!(target_os = "windows") {
                    Command::new("cmd")
                        .args(["/C", format!("code {arg}").as_str()])
                        .output()?;
                } else {
                    run_cmd!(code $arg)?;
                }
            }
            Application::Vim => {
                if cfg!(target_os = "windows") {
                    Command::new("cmd")
                        .args(["/C", format!("vim {arg}").as_str()])
                        .output()?;
                } else {
                    run_cmd!(vim $arg)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub fn from(key: &str) -> Self {
        match key.to_lowercase().as_str() {
            "vim" => Application::Vim,
            "vscode" => Application::VSCode,
            _ => Application::None,
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Application::None
    }
}

impl Display for Application {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Application::VSCode => write!(f, "{}", VSCODE_NAME),
            Application::Vim => write!(f, "{}", VIM_NAME),
            Application::Custom => write!(f, "{}", CUSTOM_NAME),
            _ => write!(f, "{}", NO_EDITOR_SET),
        }
    }
}
