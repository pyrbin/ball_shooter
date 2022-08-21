use bevy::diagnostic::Diagnostics;
use bevy::prelude::*;
use bevy_egui::*;

pub fn diagnostic_ui(ui: &mut egui::Ui, diagnostics: &Diagnostics) {
    egui::Grid::new("frame time diagnostics").show(ui, |ui| {
        for diagnostic in diagnostics.iter() {
            ui.label(diagnostic.name.as_ref());
            if let Some(average) = diagnostic.average() {
                ui.label(format!("{:.2}", average));
            }
            ui.end_row();
        }
    });
}

pub fn egui_display_diagnostics(
    mut egui_context: ResMut<EguiContext>,
    diagnostics: Res<Diagnostics>,
) {
    egui::Window::new("diagnostics")
        .min_width(0.0)
        .default_width(1.0)
        .show(egui_context.ctx_mut(), |ui| {
            diagnostic_ui(ui, &*diagnostics);
        });
}

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy::diagnostic::DiagnosticsPlugin)
            .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_system(egui_display_diagnostics);
    }
}
