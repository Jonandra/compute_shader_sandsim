use crate::DynamicSettings;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_vulkano::{
    egui_winit_vulkano::{egui, egui::Ui},
    BevyVulkanoWindows,
};

use crate::matter::MatterId;
use strum::IntoEnumIterator;

// Give our text a custom size
fn sized_text(ui: &mut Ui, text: impl Into<String>, size: f32) {
    ui.label(egui::RichText::new(text).size(size));
}

// System to generate user interface with egui
pub fn user_interface(
    vulkano_windows: NonSend<BevyVulkanoWindows>,
    diagnostics: Res<Diagnostics>,
    mut settings: ResMut<DynamicSettings>,
) {
    let (_, gui) = vulkano_windows.get_primary_window_renderer().unwrap();
    let ctx = gui.context();
    egui::Area::new("fps")
        .fixed_pos(egui::pos2(10.0, 10.0))
        .show(&ctx, |ui| {
            let size = 15.0;
            ui.heading("Info");
            if let Some(diag) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(avg) = diag.average() {
                    sized_text(ui, format!("FPS: {:.2}", avg), size);
                }
            }
            ui.heading("Settings");
            ui.add(egui::Slider::new(&mut settings.brush_radius, 0.5..=40.0).text("Brush Radius"));

            // Selectable matter
            egui::ComboBox::from_label("Matter")
                .selected_text(format!("{:?}", settings.draw_matter))
                .show_ui(ui, |ui| {
                    for matter in MatterId::iter() {
                        ui.selectable_value(
                            &mut settings.draw_matter,
                            matter,
                            format!("{:?}", matter),
                        );
                    }
                });
        });
}
