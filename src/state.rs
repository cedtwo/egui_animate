use std::any::Any;

use crate::Animation;
use crate::RunState;
use crate::mem;
use crate::ty::AnimFn;

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
pub fn animate<T, R, F0, F1>(
    ui: &mut egui::Ui,
    id: impl Into<egui::Id>,
    value: T,
    animation: Animation<F0, F1>,
    add_contents: impl FnOnce(&mut egui::Ui, T) -> R,
) where
    T: 'static + Any + Clone + Send + Sync + Default + PartialEq,
    F0: AnimFn,
    F1: AnimFn,
{
    let id: egui::Id = id.into();

    let current_time = ui.ctx().input(|input| input.time);
    let current_value = value;
    let start_value = mem::get_or_insert_start_value(ui, id, current_value.clone());

    match start_value == current_value {
        true => add_contents(ui, current_value),
        false => {
            let start_time = mem::get_or_insert_start_time(ui, id, current_time);
            let state = AnimationState::new(start_time, current_time);

            ui.ctx().request_repaint();
            animation.animate(ui, id, state, start_value, current_value, add_contents)
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
pub fn run_state(ui: &mut egui::Ui, id: impl Into<egui::Id>, animation: Animation) -> RunState {
    let id: egui::Id = id.into();

    match mem::get_start_time(ui, id) {
        Some(start_time) => {
            let current_time = ui.ctx().input(|input| input.time);
            let state = AnimationState::new(start_time, current_time);
            animation.run_state(state)
        }
        None => Default::default(),
    }
}

/// The current state of an animation.
pub(super) struct AnimationState {
    start_time: f64,
    current_time: f64,
}

impl AnimationState {
    /// Create a new `AnimationState` from the `start_time` and `current_time`.
    pub const fn new(start_time: f64, current_time: f64) -> Self {
        Self {
            start_time,
            current_time,
        }
    }

    /// Get animation start time.
    #[inline]
    pub(super) fn start(&self) -> f64 {
        self.start_time
    }

    /// Get the elapsed time. Returns `Some(0.0)` if the animation has yet to begin, and `None` if
    /// the animation has finished.
    pub(super) fn elapsed(&self, duration: f64) -> Option<f32> {
        let elapsed = (self.current_time - self.start()).max(0.0);
        (elapsed < duration).then_some(elapsed as f32)
    }

    /// Get the elapsed normal. Returns `Some(0.0)` if the animation has yet to begin, and `None` if
    /// the animation has finished.
    pub(super) fn elapsed_normal(&self, duration: f64) -> Option<f32> {
        self.elapsed(duration)
            .map(|elapsed| elapsed / duration as f32)
    }

    /// Offset the animation start time by the given amount.
    pub(super) fn offset(&self, offset: f64) -> Self {
        Self {
            start_time: self.start_time + offset,
            current_time: self.current_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod animation_state {
        use super::*;

        const TEST_ANIM_STATE: AnimationState = AnimationState::new(1.0, 1.0);

        #[test]
        fn test_elapsed() {
            let mut state = TEST_ANIM_STATE;

            assert_eq!(state.elapsed(1.0), Some(0.0));
            state.current_time = 1.5;
            assert_eq!(state.elapsed(1.0), Some(0.5));
            state.current_time = 2.0;
            assert_eq!(state.elapsed(1.0), None);
            state.current_time = 3.0;
            assert_eq!(state.elapsed(1.0), None);
        }

        #[test]
        fn test_elapsed_normal() {
            let mut state = TEST_ANIM_STATE;

            assert_eq!(state.elapsed_normal(1.0), Some(0.0));
            state.current_time = 1.75;
            assert_eq!(state.elapsed_normal(1.0), Some(0.75));
            state.current_time = 3.0;
            assert_eq!(state.elapsed_normal(1.0), None);
            state.current_time = 4.0;
            assert_eq!(state.elapsed_normal(1.0), None);
        }
    }
}
