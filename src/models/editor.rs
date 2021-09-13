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
    fn command(&self) -> &'a str {
        match self {
            EditorApp::VSCode => "code",
            EditorApp::Vim => "vim",
            EditorApp::Nano => "nano",
            EditorApp::None => "",
        }
    }
    pub fn from(key: char) -> Self {
        match key {
            'v' => EditorApp::Vim,
            'c' => EditorApp::VSCode,
            'n' => EditorApp::Nano,
            _ => EditorApp::None,
        }
    }
}
