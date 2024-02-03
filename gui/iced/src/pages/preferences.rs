use devmode_shared::{application::Application, editor::Editor, host::Host, settings::Settings};
use iced::widget::text;

use crate::app::Message;

#[derive(Debug)]
pub struct PreferencesPanel {
    settings: Settings,
}

impl Default for PreferencesPanel {
    fn default() -> Self {
        let settings = Settings::current().unwrap_or_default();
        Self { settings }
    }
}

impl PreferencesPanel {
    pub(crate) fn ui(&self) -> iced::Element<'_, Message> {
        text("Preferences").into()
    }
}
