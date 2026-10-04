use std::any::Any;
use std::ops::{Deref, DerefMut};

use crate::segment::AnimationSegment;
use crate::state::AnimationState;
use crate::ty::{AnimFn, AnimPointer};

/// An animation defined by out-in [`AnimationSegment`](s).
///
/// An animation must include either an *out* function, an *in* function, or both.
/// Single function animations may be suitable for displaying or hiding elements,
/// while out/in animations simplify transitions.
///
/// ## Example
/// ```
/// # use egui_animate::Animation;
/// // A 0.2 second fade out/in animation.
/// const ANIM: Animation = Animation::new(
///     0.2,
///     |ui, normal| ui.set_opacity(1.0 - normal),
///     |ui, normal| ui.set_opacity(normal),
/// );
/// ```
///
/// # Defining animation functions
///
/// Mutating functions recieve a *normal*, representing the linear relative
/// progress (`0.0` to `1.0`) of the animation segment. This can be passed to
/// easing functions for smoother animations, and often reversed to reverse a
/// previously defined animation.
///
/// ```
/// # use egui_animate::Animation;
/// fn out_fn(ui: &mut egui::Ui, normal: f32) {
///     // Reverse the normal (1.0 to 0.0 progression), and pass to the `in_fn`.
///     in_fn(ui, 1.0 - normal);
/// };
/// fn in_fn(ui: &mut egui::Ui, normal: f32) {
///     // Apply easing to the normal.
///     let normal = egui::emath::easing::quadratic_out(normal);
///     // Fade in, progressing from 0.0 to 1.0.
///     ui.set_opacity(normal);
/// };
///
/// const FADE_ANIM: Animation = Animation::new(0.2, out_fn, in_fn);
/// ```
#[derive(Clone, Copy)]
pub struct SingleAnimation<F>(AnimationSegment<F>);

impl<F> Deref for SingleAnimation<F> {
    type Target = AnimationSegment<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> DerefMut for SingleAnimation<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SingleAnimation<AnimPointer> {
    /// An empty placeholder animation.
    pub const EMPTY: Self = SingleAnimation::from_segment(AnimationSegment::EMPTY);
}

impl<F> SingleAnimation<F> {
    /// Create a new `Animation` with the given total `duration`, split over segments.
    pub const fn new(duration: f32, f: F) -> Self {
        Self(AnimationSegment::new(duration, f))
    }

    /// Create a new `Animation` from the given [`AnimationSegment`]s.
    pub const fn from_segment(seg: AnimationSegment<F>) -> Self {
        Self(seg)
    }

    /// Get the total duration of the animation.
    pub const fn duration(&self) -> f32 {
        self.0.duration
    }

    // /// Get the `RunState` for the current frame.
    // pub(super) fn run_state(&self, state: AnimationState) -> RunState {
    //     if let Some(normal) = state.elapsed_normal(self.out_dur() as f64) {
    //         RunState::OutSeg(normal)
    //     } else if let Some(normal) = state
    //         .offset(self.out_dur() as f64)
    //         .elapsed_normal(self.in_dur() as f64)
    //     {
    //         RunState::InSeg(normal)
    //     } else {
    //         RunState::None
    //     }
    // }

    /// Call the `AnimationSegment` for the current frame.
    pub(super) fn animate<T: 'static + Any + Clone + Send + Sync + Default, R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        state: AnimationState,
        start_value: T,
        current_value: T,
        add_contents: impl FnOnce(&mut egui::Ui, T) -> R,
    ) -> R
    where
        F: AnimFn,
    {
        self.0
            .animate(ui, id, normal, |ui| add_contents(ui, start_value))
        // match self.run_state(state) {
        //     RunState::OutSeg(normal) => {
        //         self.animate_out(ui, id, normal, |ui| add_contents(ui, start_value))
        //     }
        //     RunState::InSeg(normal) => {
        //         mem::clear_animation_layer(ui, id);
        //         self.animate_in(ui, id, normal, |ui| add_contents(ui, current_value))
        //     }
        //     RunState::None => {
        //         mem::clear_start_value::<T>(ui, id);
        //         mem::clear_start_time(ui, id);
        //         mem::clear_animation_layer(ui, id);

        //         add_contents(ui, current_value)
        //     }
        // }
    }
}

impl Default for SingleAnimation<AnimPointer> {
    fn default() -> Self {
        Self(Default::default())
    }
}
