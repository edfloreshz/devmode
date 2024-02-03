use devmode_shared::{application::Application, editor::Editor, host::Host, settings::Settings};
use eframe::{
    egui::{ComboBox, Response, TextEdit, Ui},
    epaint::vec2,
};

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
    pub(crate) fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.heading("Preferences");
        ui.separator();
        ui.style_mut().spacing.item_spacing = vec2(10.0, 10.0);
        ui.strong("Git service");
        ui.horizontal(|ui| {
            ComboBox::from_label("Select your prefered git service provider.")
                .selected_text(self.settings.host.clone())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.settings.host,
                        Host::GitHub.to_string(),
                        "GitHub",
                    );
                    ui.selectable_value(
                        &mut self.settings.host,
                        Host::GitLab.to_string(),
                        "GitLab",
                    );
                });
        });
        ui.strong("Git username");
        ui.horizontal(|ui| {
            ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.settings.owner).margin(vec2(10.0, 10.0)),
            )
        });
        ui.strong("Editor");
        ui.horizontal(|ui| {
            ComboBox::from_label("Select your favorite editor.")
                .selected_text(self.settings.editor.app.clone().to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.settings.editor,
                        Editor::new(Application::VSCode),
                        Application::VSCode.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.settings.editor,
                        Editor::new(Application::Vim),
                        Application::Vim.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.settings.editor,
                        Editor::new(Application::Custom),
                        Application::Custom.to_string(),
                    );
                });
        });
        ui.separator()
    }

    pub(crate) fn footer(&mut self, ui: &mut Ui) -> Response {
        ui.button("Save")
    }
}
