use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct Editor {
    pub app: EditorApp,
}

impl Editor {
    pub fn new(app: EditorApp) -> Self {
        Editor { app }
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

impl<'a> EditorApp {
    pub fn from(key: &str) -> Self {
        match key.to_lowercase().as_str() {
            "vim" => EditorApp::Vim,
            "vscode" => EditorApp::VSCode,
            "nano" => EditorApp::Nano,
            _ => EditorApp::None,
        }
    }
}
