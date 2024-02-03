use eframe::egui::{Layout, Response, ScrollArea, Ui};

#[derive(Debug)]
pub struct WorkspacesPanel {
    workspaces: Vec<Workspace>,
    selected: Workspace,
}

impl Default for WorkspacesPanel {
    fn default() -> Self {
        let workspaces: Vec<Workspace> = (0..20)
            .map(|i| Workspace {
                name: i.to_string(),
            })
            .collect();
        Self {
            workspaces: workspaces.clone(),
            selected: workspaces.first().unwrap().to_owned(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Workspace {
    name: String,
}

impl WorkspacesPanel {
    pub(crate) fn ui(&mut self, ui: &mut Ui) -> Response {
        ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Workspaces");
            ui.separator();
            ui.vertical(|ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    // Use the `Layout` API to justify the content vertically
                    ui.with_layout(
                        Layout::top_down(eframe::emath::Align::Min).with_cross_justify(true),
                        |ui| {
                            for ws in &self.workspaces {
                                let name = ws.name.clone();
                                ui.selectable_value(&mut self.selected, ws.clone(), &name);
                            }
                        },
                    );
                })
            });
        });
        ui.separator()
    }

    pub(crate) fn footer(&mut self, ui: &mut Ui) -> Response {
        ui.button("Add");
        ui.button("Edit");
        ui.button("Remove")
    }
}
