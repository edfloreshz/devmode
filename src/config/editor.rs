use crate::config::editor_app::EditorApp;
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
    pub fn custom(command: String) -> Self {
        Editor {
            app: EditorApp::Custom,
            command,
        }
    }
}
