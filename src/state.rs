use std::any::Any;

use crate::Animation;
use crate::mem;

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
pub fn animate<T: 'static + Any + Clone + Send + Sync + Default + PartialEq, R>(
    ui: &mut egui::Ui,
    id: impl Into<egui::Id>,
    value: T,
    animation: Animation,
    add_contents: impl FnOnce(&mut egui::Ui, T) -> R,
) {
    let id: egui::Id = id.into();

    let current_time = ui.ctx().input(|input| input.time);
    let current_value = value;
    let start_value = mem::get_start_value(ui, id, current_value.clone());

    match start_value == current_value {
        true => add_contents(ui, current_value),
        false => {
            let start_time = mem::get_start_time(ui, id, current_time);
            let animation = AnimationState::new(start_time, current_time, animation);

            ui.ctx().request_repaint();
            animation.animate(ui, id, start_value, current_value, add_contents)
        }
    };
}

/// The current state of an animation. Defines an animation scope, delegating variables
/// to the currently progressing animation.
struct AnimationState {
    start_time: f64,
    current_time: f64,

    animation: Animation,
}

impl AnimationState {
    /// Create a new `AnimationState` from the `start_time`, `current_time` and `Animation`.
    pub const fn new(start_time: f64, current_time: f64, animation: Animation) -> Self {
        Self {
            start_time,
            current_time,
            animation,
        }
    }

    /// Get the **out** segment duration.
    #[inline]
    fn out_dur(&self) -> f32 {
        self.animation.out_seg.duration
    }

    /// Get the **out** segment start time.
    #[inline]
    fn out_start(&self) -> f64 {
        self.start_time
    }

    /// Get the **out** segment end time.
    #[inline]
    fn out_end(&self) -> f64 {
        self.out_start() + self.out_dur() as f64
    }

    /// Get the elapsed time of the **out** segment. Returns `Some(0.0)` if the animation
    /// has yet to begin, and `None` if the animation has finished.
    fn out_elapsed(&self) -> Option<f32> {
        let out_elapsed = (self.current_time - self.out_start()).max(0.0) as f32;
        (out_elapsed < self.out_dur()).then_some(out_elapsed)
    }

    /// Get the elapsed normal of the **out** segment. Returns `Some(0.0)` if the animation
    /// has yet to begin, and `None` if the animation has finished.
    fn out_elapsed_normal(&self) -> Option<f32> {
        self.out_elapsed().map(|elapsed| elapsed / self.out_dur())
    }

    /// Get the **in** segment duration.
    #[inline]
    fn in_dur(&self) -> f32 {
        self.animation.in_seg.duration
    }

    /// Get the **in** segment start time.
    #[inline]
    fn in_start(&self) -> f64 {
        self.out_end()
    }

    /// Get the **in** segment end time.
    #[allow(dead_code)]
    #[inline]
    fn in_end(&self) -> f64 {
        self.in_start() + self.in_dur() as f64
    }

    /// Get the elapsed time of the **in** segment. Returns `Some(0.0)` if the animation
    /// has yet to begin, and `None` if the animation has finished.
    fn in_elapsed(&self) -> Option<f32> {
        let in_elapsed = (self.current_time - self.in_start()).max(0.0) as f32;
        (in_elapsed < self.in_dur()).then_some(in_elapsed)
    }

    /// Get the elapsed normal of the **in** segment. Returns `Some(0.0)` if the animation
    /// has yet to begin, and `None` if the animation has finished.
    fn in_elapsed_normal(&self) -> Option<f32> {
        self.in_elapsed().map(|elapsed| elapsed / self.in_dur())
    }

    /// Call the `AnimationSegment` for thec
    fn animate<T: 'static + Any + Clone + Send + Sync + Default, R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        start_value: T,
        current_value: T,
        add_contents: impl FnOnce(&mut egui::Ui, T) -> R,
    ) -> R {
        if let Some(normal) = self.out_elapsed_normal() {
            self.animate_out(ui, id, normal, |ui| add_contents(ui, start_value))
        } else if let Some(normal) = self.in_elapsed_normal() {
            mem::clear_animation_layer(ui, id);
            self.animate_in(ui, id, normal, |ui| add_contents(ui, current_value))
        } else {
            mem::clear_start_value::<T>(ui, id);
            mem::clear_start_time(ui, id);
            mem::clear_animation_layer(ui, id);

            add_contents(ui, current_value)
        }
    }

    /// Delegate to the **out** segment [`AnimationSegment::animate`] fn.
    #[inline]
    fn animate_out<R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        normal: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        self.animation.out_seg.animate(ui, id, normal, add_contents)
    }

    /// Delegate to the **in** segment [`AnimationSegment::animate`] fn.
    #[inline]
    fn animate_in<R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        normal: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        self.animation.in_seg.animate(ui, id, normal, add_contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod animation_state {
        use super::*;

        const TEST_ANIM_STATE: AnimationState = AnimationState::new(
            1.0,
            1.0,
            Animation {
                out_seg: crate::AnimationSegment {
                    duration: 1.5,
                    anim_fn: |_, _| {},
                },
                in_seg: crate::AnimationSegment {
                    duration: 1.5,
                    anim_fn: |_, _| {},
                },
            },
        );

        #[test]
        fn test_out_end() {
            let state = TEST_ANIM_STATE;
            assert_eq!(state.out_end(), 2.5);
        }

        #[test]
        fn test_out_elapsed() {
            let mut state = TEST_ANIM_STATE;

            assert_eq!(state.out_elapsed(), Some(0.0));
            state.current_time = 2.0;
            assert_eq!(state.out_elapsed(), Some(1.0));
            state.current_time = 3.0;
            assert_eq!(state.out_elapsed(), None);
            state.current_time = 4.0;
            assert_eq!(state.out_elapsed(), None);
        }
        #[test]
        fn test_out_elapsed_normal() {
            let mut state = TEST_ANIM_STATE;

            assert_eq!(state.out_elapsed_normal(), Some(0.0));
            state.current_time = 1.75;
            assert_eq!(state.out_elapsed_normal(), Some(0.5));
            state.current_time = 3.0;
            assert_eq!(state.out_elapsed_normal(), None);
            state.current_time = 4.0;
            assert_eq!(state.out_elapsed_normal(), None);
        }

        #[test]
        fn test_in_end() {
            let state = TEST_ANIM_STATE;
            assert_eq!(state.in_end(), 4.0);
        }

        #[test]
        fn test_in_elapsed() {
            let mut state = TEST_ANIM_STATE;

            assert_eq!(state.in_elapsed(), Some(0.0));
            state.current_time = 2.0;
            assert_eq!(state.in_elapsed(), Some(0.0));
            state.current_time = 3.0;
            assert_eq!(state.in_elapsed(), Some(0.5));
            state.current_time = 4.0;
            assert_eq!(state.in_elapsed(), None);
            state.current_time = 5.0;
            assert_eq!(state.in_elapsed(), None);
        }
        #[test]
        fn test_in_elapsed_normal() {
            let mut state = TEST_ANIM_STATE;

            assert_eq!(state.in_elapsed_normal(), Some(0.0));
            state.current_time = 2.0;
            assert_eq!(state.in_elapsed_normal(), Some(0.0));
            state.current_time = 3.25;
            assert_eq!(state.in_elapsed_normal(), Some(0.5));
            state.current_time = 4.0;
            assert_eq!(state.in_elapsed_normal(), None);
            state.current_time = 5.0;
            assert_eq!(state.in_elapsed_normal(), None);
        }
    }
}
