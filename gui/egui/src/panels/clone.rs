use eframe::{
    egui::{Layout, Response, ScrollArea, TextEdit, Ui},
    epaint::vec2,
};

use crate::Repository;

#[derive(Debug)]
pub struct ClonePanel {
    repositories: Vec<Repository>,
    selected: Repository,
    url: String,
}

impl Default for ClonePanel {
    fn default() -> Self {
        let repositories: Vec<Repository> = (0..20)
            .map(|i| Repository {
                name: "Test".to_string(),
                url: i.to_string(),
            })
            .collect();
        Self {
            repositories: repositories.clone(),
            selected: repositories.first().unwrap().to_owned(),
            url: String::default(),
        }
    }
}

impl ClonePanel {
    pub(crate) fn ui(&mut self, ui: &mut Ui) -> Response {
        ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Remote repositories");
            ui.separator();
            ui.vertical(|ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    // Use the `Layout` API to justify the content vertically
                    ui.with_layout(
                        Layout::top_down(eframe::emath::Align::Min).with_cross_justify(true),
                        |ui| {
                            for repo in &self.repositories {
                                let name = repo.name.clone();
                                ui.selectable_value(&mut self.selected, repo.clone(), &name);
                            }
                        },
                    );
                })
            });
        });
        ui.separator()
    }

    pub(crate) fn footer(&mut self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label("URL:");
            ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.url).margin(vec2(10.0, 10.0)),
            );
        });
        ui.button("Clone")
    }
}
