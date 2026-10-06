use egui::{Pos2, Vec2};

use crate::mem;

/// Set the [`egui::Ui`] opacity to the given `normal` value.
#[inline]
pub fn fade(ui: &mut egui::Ui, normal: f32) {
    ui.set_opacity(normal);
}

/// Offset the [`egui::Ui`] transform to the given `target`. This overrides any prior translation
/// and scale.
#[inline]
pub fn translate(ui: &mut egui::Ui, normal: f32, target: Vec2) {
    mem::set_transform_layer_translation(ui, ui.layer_id(), target - target * normal);
}

/// Set the [`egui::Ui`] scale to the given `scale`. Requires the center position to scale from.
/// Consider using `ui.max_rect().center()`, or offsetting from that value. This overrides any prior
/// translation and scale.
#[inline]
pub fn scale(ui: &mut egui::Ui, normal: f32, center: Pos2, scale: f32) {
    let center = center * (1.0 - scale);

    let translation = center - center * normal;
    let scaling = 1.0 + (normal - 1.0) * (1.0 - scale);

    mem::set_transform_layer_translation(ui, ui.layer_id(), translation);
    mem::set_transform_layer_scale(ui, ui.layer_id(), scaling);
}
