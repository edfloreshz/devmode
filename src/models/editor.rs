use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct Editor {
    pub app: EditorApp,
    pub command: String,
}

impl Editor {
    pub fn new(app: EditorApp) -> Self {
        let command = app.command();
        Editor { app, command }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum EditorApp {
    VSCode,
    Vim,
    Nano,
    None,
}

impl Default for EditorApp {
    fn default() -> Self {
        EditorApp::None
    }
}

impl EditorApp {
    pub fn command(&self) -> String {
        String::from(match self {
            EditorApp::VSCode => "code",
            EditorApp::Vim => "vim",
            EditorApp::Nano => "nano",
            EditorApp::None => "",
        })
    }
    pub fn from(key: &str) -> Self {
        match key.to_lowercase().as_str() {
            "vim" => EditorApp::Vim,
            "vscode" => EditorApp::VSCode,
            "nano" => EditorApp::Nano,
            _ => EditorApp::None,
        }
    }
}
