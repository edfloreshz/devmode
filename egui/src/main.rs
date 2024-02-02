mod panels;

use eframe::{
    egui::{CentralPanel, Frame, TopBottomPanel},
    epaint::vec2,
    run_native, App, NativeOptions,
};

use panels::*;

#[derive(Default)]
struct Devmode {
    open_panel: Panel,
    clone: ClonePanel,
    open: OpenPanel,
    workspaces: WorkspacesPanel,
    preferences: PreferencesPanel,
}

#[derive(Default, PartialEq)]
enum Panel {
    #[default]
    Clone,
    Open,
    Workspaces,
    Preferences,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Repository {
    name: String,
    url: String,
}

impl App for Devmode {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        // catppuccin_egui::set_theme(&ctx, catppuccin_egui::MOCHA);
        ctx.style_mut(|s| {
            s.spacing.button_padding = vec2(20.0, 10.0);
        });

        TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = vec2(10.0, 10.0);
            Frame::central_panel(ui.style()).show(ui, |ui| {
                ui.vertical_centered_justified(|ui| match self.open_panel {
                    Panel::Clone => self.clone.footer(ui),
                    Panel::Open => self.open.footer(ui),
                    Panel::Workspaces => self.workspaces.footer(ui),
                    Panel::Preferences => self.preferences.footer(ui),
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.open_panel, Panel::Clone, "Clone");
                ui.selectable_value(&mut self.open_panel, Panel::Open, "Open");
                ui.selectable_value(&mut self.open_panel, Panel::Workspaces, "Workspaces");
                ui.selectable_value(&mut self.open_panel, Panel::Preferences, "Preferences");
            });
            ui.separator();
            match self.open_panel {
                Panel::Clone => self.clone.ui(ui),
                Panel::Open => self.open.ui(ui),
                Panel::Workspaces => self.workspaces.ui(ui),
                Panel::Preferences => self.preferences.ui(ui),
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        vsync: true,
        follow_system_theme: true,
        window_builder: Some(Box::new(|vp| vp.with_min_inner_size(vec2(400.0, 300.0)))),
        centered: true,
        ..Default::default()
    };
    run_native("Devmode", options, Box::new(|_| Box::<Devmode>::default()))
}
