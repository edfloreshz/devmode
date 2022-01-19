use serde::{Deserialize, Serialize};

use crate::config::application::Application;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct Editor {
    pub app: Application,
    pub command: String,
}

impl Editor {
    pub fn new(app: Application) -> Self {
        let command = app.command();
        Editor { app, command }
    }
    pub fn custom(command: String) -> Self {
        Editor {
            app: Application::Custom,
            command,
        }
    }
}
