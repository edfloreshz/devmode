use eframe::egui::{Response, Ui};

#[derive(Debug, Default)]
pub struct WorkspacesPanel;

impl WorkspacesPanel {
    pub(crate) fn ui(&self, ui: &mut Ui) -> Response {
        ui.label("Workspaces")
    }

    pub(crate) fn footer(&mut self, ui: &mut Ui) -> Response {
        ui.label("Footer")
    }
}
