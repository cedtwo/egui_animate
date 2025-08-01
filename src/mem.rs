//! Functions that interact with `egui` persistant memory.
use std::any::Any;

use crate::AnimationSegment;

const START_TIME_SUFFIX: &'static str = "start_time";
const START_VALUE_SUFFIX: &'static str = "start_value";

pub(super) fn get_start_time(ui: &mut egui::Ui, id: egui::Id, current_time: f64) -> f64 {
    ui.ctx().memory_mut(|m| {
        *m.data
            .get_temp_mut_or_insert_with(id.with(START_TIME_SUFFIX), || current_time)
    })
}

pub(super) fn clear_start_time(ui: &mut egui::Ui, id: egui::Id) -> Option<f64> {
    ui.ctx()
        .memory_mut(|m| m.data.remove_temp(id.with(START_TIME_SUFFIX)))
}

pub(super) fn get_start_value<T: 'static + Any + Clone + Send + Sync>(
    ui: &mut egui::Ui,
    id: egui::Id,
    current_value: T,
) -> T {
    ui.ctx().memory_mut(|m| {
        m.data
            .get_temp_mut_or_insert_with(id.with(START_VALUE_SUFFIX), || current_value)
            .clone()
    })
}

pub(super) fn clear_start_value<T: 'static + Any + Clone + Send + Sync + Default>(
    ui: &mut egui::Ui,
    id: egui::Id,
) -> Option<T> {
    ui.ctx()
        .memory_mut(|m| m.data.remove_temp(id.with(START_VALUE_SUFFIX)))
}

pub(super) fn clear_animation_layer(
    ui: &mut egui::Ui,
    id: egui::Id,
) -> Option<egui::emath::TSTransform> {
    let layer_id = AnimationSegment::animation_layer(ui, id);
    ui.memory_mut(|m| m.to_global.remove(&layer_id))
}
