use std::any::Any;

use crate::animate::Animate;
use crate::mem;
use crate::segment::AnimationSegment;
use crate::state::{AnimProgress, AnimValues};
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
pub struct Animation<F0 = AnimPointer, F1 = AnimPointer> {
    /// The segment animating the prior value **out**.
    pub out_seg: AnimationSegment<F0>,
    /// The segment animating the new value **in**.
    pub in_seg: AnimationSegment<F1>,
}

impl Animation<AnimPointer, AnimPointer> {
    /// An empty placeholder animation.
    pub const EMPTY: Self =
        Animation::from_segments(AnimationSegment::EMPTY, AnimationSegment::EMPTY);
}

impl<F0> Animation<F0, AnimPointer> {
    /// Create a new `Animation` with only the *out* segment. Passes the the prior
    /// value to the animation scope for the duration of the `out_fn`.
    pub const fn new_out(duration: f32, out_fn: F0) -> Self {
        let out_seg = AnimationSegment::new(duration, out_fn);
        let in_seg = AnimationSegment::EMPTY;

        Self { out_seg, in_seg }
    }
}

impl<F1> Animation<AnimPointer, F1> {
    /// Create a new `Animation` with only the *in* segment. Passes the the mutated
    /// value to the animation scope for the duration of the `in_fn`.
    pub const fn new_in(duration: f32, in_fn: F1) -> Self {
        let out_seg = AnimationSegment::EMPTY;
        let in_seg = AnimationSegment::new(duration, in_fn);

        Self { out_seg, in_seg }
    }
}

impl<F0, F1> Animation<F0, F1> {
    /// Create a new `Animation` with the given total `duration`, split over segments.
    pub const fn new(duration: f32, out_fn: F0, in_fn: F1) -> Self {
        let segment_duration = duration / 2.0;

        let out_seg = AnimationSegment::new(segment_duration, out_fn);
        let in_seg = AnimationSegment::new(segment_duration, in_fn);

        Self { out_seg, in_seg }
    }

    /// Create a new `Animation` from the given [`AnimationSegment`]s.
    pub const fn from_segments(
        out_seg: AnimationSegment<F0>,
        in_seg: AnimationSegment<F1>,
    ) -> Self {
        Self { out_seg, in_seg }
    }

    /// Get the **out** segment duration.
    #[inline]
    fn out_dur(&self) -> f32 {
        self.out_seg.duration
    }

    /// Get the **in** segment duration.
    #[inline]
    fn in_dur(&self) -> f32 {
        self.in_seg.duration
    }

    /// Get the total duration of the animation.
    pub const fn duration(&self) -> f32 {
        self.out_seg.duration + self.in_seg.duration
    }

    /// Delegate to the **out** segment [`AnimationSegment::animate`] fn.
    #[inline]
    fn animate_out<R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        normal: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R
    where
        F0: AnimFn,
    {
        self.out_seg.animate(ui, id, normal, add_contents)
    }

    /// Delegate to the **in** segment [`AnimationSegment::animate`] fn.
    #[inline]
    fn animate_in<R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        normal: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R
    where
        F1: AnimFn,
    {
        self.in_seg.animate(ui, id, normal, add_contents)
    }
}

impl<F0: AnimFn, F1: AnimFn> Animate for Animation<F0, F1> {
    type RunState = RunState;

    fn run_state(&self, progress: AnimProgress) -> Self::RunState {
        if let Some(normal) = progress.elapsed_normal(self.out_dur() as f64) {
            RunState::OutSeg(normal)
        } else if let Some(normal) = progress
            .offset(self.out_dur() as f64)
            .elapsed_normal(self.in_dur() as f64)
        {
            RunState::InSeg(normal)
        } else {
            RunState::None
        }
    }

    fn animate<T: 'static + Any + Clone + Send + Sync + Default, R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        progress: AnimProgress,
        vars: AnimValues<T>,
        add_contents: impl FnOnce(&mut egui::Ui, T) -> R,
    ) -> R {
        match self.run_state(progress) {
            RunState::OutSeg(normal) => {
                self.animate_out(ui, id, normal, |ui| add_contents(ui, vars.start_value()))
            }
            RunState::InSeg(normal) => {
                mem::clear_animation_layer(ui, id);
                self.animate_in(ui, id, normal, |ui| add_contents(ui, vars.current_value()))
            }
            RunState::None => {
                mem::clear_start_value::<T>(ui, id);
                mem::clear_start_time(ui, id);
                mem::clear_animation_layer(ui, id);

                add_contents(ui, vars.current_value())
            }
        }
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            out_seg: Default::default(),
            in_seg: Default::default(),
        }
    }
}

/// An identified animation segment and *normal*.
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub enum RunState {
    /// The *out* animation segment normal.
    OutSeg(f32),
    /// The *in* animation segment normal.
    InSeg(f32),
    /// The animation is not currently running.
    #[default]
    None,
}

impl RunState {
    /// Returns `true` if the animation is in either the *out* or *in* state.
    pub fn is_running(&self) -> bool {
        match self {
            RunState::OutSeg(_) | RunState::InSeg(_) => true,
            RunState::None => false,
        }
    }
}
