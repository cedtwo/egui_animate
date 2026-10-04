use std::any::Any;

use crate::animate::Animate;
use crate::mem;
use crate::state::AnimProgress;
use crate::state::AnimValues;

/// Create an animation that transitions between changes of the given `value`.
///
/// Requires a unique [`egui::Id`], and [`Animation`]. See [`Animation`] for details
/// on how to define an animation.
///
/// # Example
/// ```
/// # use egui;
/// # use eframe;
/// # use egui_animate::*;
/// // A linear 0.3 second fade out/in animation.
/// const FADE_ANIM: Animation = Animation::new(
///     0.3,
///     |ui, normal| ui.set_opacity(1.0 - normal),
///     |ui, normal| ui.set_opacity(normal),
/// );
///
/// // The variable state.
/// let mut my_state: u32 = 0;
///
/// # let ctx = egui::Context::default();
/// #
/// # ctx.run(egui::RawInput::default(), |ctx| {
/// # egui::CentralPanel::default().show(ctx, |ui| {
/// #
/// // An animation that triggers on button press.
/// animate(ui, "my_fade", my_state, FADE_ANIM, |ui, value| {
///     if ui.button(format!("Value is {}", value)).clicked() {
///         my_state += 1;
///     };
/// });
/// #
/// # });
/// # });
/// ```
pub fn animate<T, R, A>(
    ui: &mut egui::Ui,
    id: impl Into<egui::Id>,
    value: T,
    anim: A,
    add_contents: impl FnOnce(&mut egui::Ui, T) -> R,
) where
    T: 'static + Any + Clone + Send + Sync + Default + PartialEq,
    A: Animate,
{
    let id: egui::Id = id.into();

    let current_time = ui.ctx().input(|input| input.time);
    let current_value = value;
    let start_value = mem::get_or_insert_start_value(ui, id, current_value.clone());

    match start_value == current_value {
        true => add_contents(ui, current_value),
        false => {
            let start_time = mem::get_or_insert_start_time(ui, id, current_time);
            let state = AnimProgress::new(start_time, current_time);
            let vars = AnimValues::new(start_value, current_value);

            ui.ctx().request_repaint();
            anim.animate(ui, id, state, vars, add_contents)
        }
    };
}

/// Get the [`RunState`] for the animation of the given `id`. Returns `RunState::None`
/// for animations that do not exist.
///
/// # Example
/// ```
/// # use egui;
/// # use eframe;
/// # use egui_animate::*;
/// # const MY_ANIM: Animation = Animation::EMPTY;
/// # let mut my_state: u32 = 0;
/// #
/// # let ctx = egui::Context::default();
/// #
/// # ctx.run(egui::RawInput::default(), |ctx| {
/// # egui::CentralPanel::default().show(ctx, |ui| {
/// #
/// // Define an animation with a unique `id`.
/// animate(ui, "my_anim", my_state, MY_ANIM, |ui, value| {
///     // ...
/// });
///
/// // Render a ui label if the animation is running.
/// if run_state(ui, "my_anim", MY_ANIM).is_running() {
///     ui.label("Animation running...");
/// }
/// #
/// # });
/// # });
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
