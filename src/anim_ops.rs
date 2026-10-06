use std::any::Any;

use crate::anim::scope_content;
use crate::animate::Animate;
use crate::mem;
use crate::state::AnimProgress;
use crate::state::AnimValues;

/// Animate on change of the given `value`.
///
/// `animate` applies a given [`Animation`](crate::anim::Animation) to the scoped `add_contents` on
/// change of the given `value`. `animate` requires specifying a unique [`egui::Id`] for the scoped
/// variables within. See [`Animation`](crate::anim::Animation) and the crate level documentation
/// for more.
///
/// # Example
///
/// ```
/// # use egui;
/// # use eframe;
/// # use egui_animate::prelude::*;
/// // A linear sequential 0.3 second fade out/in animation.
/// const FADE_ANIM: Animation<Sequence> = Animation::new_sequence(0.3, fade, fade);
///
/// // The variable state.
/// let mut my_state: u32 = 0;
///
/// # let ctx = egui::Context::default();
/// # ctx.run_ui(egui::RawInput::default(), |ui| {
/// // An animation transitions out the prior value then transitions in the new value.
/// animate(ui, "my_fade", my_state, FADE_ANIM, |ui, value| {
///     if ui.button(format!("Value is {}", value)).clicked() {
///         my_state += 1;
///     };
/// });
/// # }).textures_delta.clear();
/// ```
pub fn animate<T, R, A>(
    ui: &mut egui::Ui,
    id: impl Into<egui::Id>,
    value: T,
    anim: A,
    mut add_contents: impl FnMut(&mut egui::Ui, T) -> R,
) -> R
where
    T: 'static + Any + Clone + Send + Sync + Default + PartialEq,
    A: Animate,
{
    let id: egui::Id = id.into();

    let current_time = ui.ctx().input(|input| input.time);
    let current_value = value;
    let start_value = mem::get_or_insert_start_value(ui, id, current_value.clone());

    match start_value == current_value {
        true => scope_content(ui, ui.layer_id(), ui.available_rect_before_wrap(), |ui| {
            add_contents(ui, current_value)
        }),
        false => {
            let start_time = mem::get_or_insert_start_time(ui, id, current_time);
            let state = AnimProgress::new(start_time, current_time);
            let vars = AnimValues::new(start_value, current_value);

            ui.ctx().request_repaint();
            anim.animate(ui, id, state, vars, add_contents)
        }
    }
}

/// Get the [`RunState`](Animate::RunState) for the animation of the given `id`. This is useful for
/// checking if an animation is running *outside* of the animation scope (eg. for disabling buttons
/// that may mutate the animation value during animation).
///
/// # Example
/// ```
/// # use egui;
/// # use eframe;
/// # use egui_animate::prelude::*;
/// # const MY_ANIM: Animation<Sequence> = Animation::EMPTY;
/// #
/// # let mut my_state: u32 = 0;
/// #
/// # let ctx = egui::Context::default();
/// # ctx.run_ui(egui::RawInput::default(), |ui| {
/// // Define an animation with a unique `id`.
/// animate(ui, "my_anim", my_state, MY_ANIM, |ui, value| {
///     // ...
/// });
///
/// // Render a ui label if the animation is running.
/// if run_state(ui, "my_anim", MY_ANIM).is_running() {
///     ui.label("Animation running...");
/// }
/// # }).textures_delta.clear();
/// ```
pub fn run_state<A: Animate>(
    ui: &mut egui::Ui,
    id: impl Into<egui::Id>,
    animation: A,
) -> A::RunState {
    let id: egui::Id = id.into();

    match mem::get_start_time(ui, id) {
        Some(start_time) => {
            let current_time = ui.ctx().input(|input| input.time);
            let state = AnimProgress::new(start_time, current_time);
            animation.run_state(state)
        }
        None => Default::default(),
    }
}
