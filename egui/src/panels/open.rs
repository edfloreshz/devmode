use eframe::egui::{Response, Ui};

#[derive(Debug, Default)]
pub struct OpenPanel;

impl OpenPanel {
    pub(crate) fn ui(&self, ui: &mut Ui) -> Response {
        ui.label("Open")
    }

    pub(crate) fn footer(&mut self, ui: &mut Ui) -> Response {
        ui.label("Footer")
    }
}
