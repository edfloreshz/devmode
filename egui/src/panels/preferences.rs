use eframe::egui::{Response, Ui};

#[derive(Debug, Default)]
pub struct PreferencesPanel;

impl PreferencesPanel {
    pub(crate) fn ui(&self, ui: &mut Ui) -> Response {
        ui.label("Preferences")
    }

    pub(crate) fn footer(&mut self, ui: &mut Ui) -> Response {
        ui.label("Footer")
    }
}
