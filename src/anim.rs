use std::any::Any;
use std::marker::PhantomData;

use crate::animate::Animate;
use crate::mem;
use crate::schedule::{Layer, Sequence};
use crate::segment::{AnimFn, AnimationSegment};
use crate::state::{AnimProgress, AnimValues};

/// # Animation
///
/// An animation defined by [`AnimationSegment`]\(s).
///
/// An animation mutates [`egui::Ui`] state for a given duration. It contains functions that recieve
/// a *normal*, the linear normalized progression of an animation segment for the segment duration.
/// An animation contains either an *out* function receiving a normal value of `1.0` to `0.0`, an
/// *in* function receiving a normal of `0.0` to `1.0` or both.
///
/// The respective *out* and *in* segments are intended for hiding and revealing [`egui::Ui`] state,
/// either individually or during a transition. `Animation` supports running each segment
/// sequentially or at the same time by passing either [`Sequence`] or [`Layer`] as the first
/// generic argument `S`. See the respective documentation of each marker type for more.
///
/// ## Examples
///
/// Animations can be customized by passing a function (or closure) with the type signature
/// `Fn(&mut egui::Ui, f32)`. Simple linear animations can pass one of the helper functions in
/// [`crate::norm_ops`]:
///
/// ```
/// # use egui_animate::prelude::*;
/// // A simple linear sequential fade in/out animation.
/// const ANIM: Animation<Sequence> = Animation::new(0.2, fade, fade);
/// ```
///
/// Providing our own function (or closure) allows for much more customization. The following
/// demonstrates applying easing to the *normal* before passing it to the [`fade`](crate::norm_ops::fade)
/// and [`translate`](crate::norm_ops::translate) functions:
///
/// ```
/// # use egui_animate::prelude::*;
/// # use egui::Vec2;
/// fn anim_fn(ui: &mut egui::Ui, normal: f32) {
///     // Use an easing function provided by `egui`.
///     let normal = egui::emath::easing::quadratic_out(normal);
///
///     // Slides and fades out to the left when used as an *out* fn.
///     // Slides and fades in from the left when used as an *in* fn.
///     translate(ui, normal, Vec2::new(-20.0, 0.0));
///     fade(ui, normal);
/// }
///
/// const ANIM: Animation<Sequence> = Animation::new(0.2, anim_fn, anim_fn);
/// ```
///
/// Note that all operations are applied to a new [`egui::Ui`] scope when calling [`animate`](crate::anim_ops::animate).
/// Animation functions can leverage this to make modification to the `Ui` that will not affect the
/// containing `Ui` elements. See [`animate`](crate::anim_ops::animate) for more on usage.
#[derive(Debug, Clone, Copy)]
pub struct Animation<S, F0 = AnimFn, F1 = AnimFn> {
    /// The segment animating the prior value **out**.
    pub out_seg: AnimationSegment<F0>,
    /// The segment animating the new value **in**.
    pub in_seg: AnimationSegment<F1>,
    /// The animation schedule (either [`Sequence`] or [`Layer`]).
    pub _schedule: PhantomData<S>,
}

impl<S> Animation<S, AnimFn, AnimFn> {
    /// An empty placeholder animation.
    pub const EMPTY: Self =
        Animation::from_segments(AnimationSegment::EMPTY, AnimationSegment::EMPTY);
}

impl<F0> Animation<Sequence, F0, AnimFn> {
    /// Create a new `Animation` with only the *out* segment. Passes the the prior value to the
    /// animation scope for the duration of the `out_fn`.
    pub const fn new_out(duration: f32, out_fn: F0) -> Self {
        let out_seg = AnimationSegment::new(duration, out_fn);
        let in_seg = AnimationSegment::EMPTY;

        Self {
            out_seg,
            in_seg,
            _schedule: PhantomData,
        }
    }
}

impl<F1> Animation<Sequence, AnimFn, F1> {
    /// Create a new `Animation` with only the *in* segment. Passes the the mutated value to the
    /// animation scope for the duration of the `in_fn`.
    pub const fn new_in(duration: f32, in_fn: F1) -> Self {
        let out_seg = AnimationSegment::EMPTY;
        let in_seg = AnimationSegment::new(duration, in_fn);

        Self {
            out_seg,
            in_seg,
            _schedule: PhantomData,
        }
    }
}

impl<S, F0, F1> Animation<S, F0, F1> {
    /// Create a new `Animation` with the given total `duration`, split over segments.
    pub const fn new(duration: f32, out_fn: F0, in_fn: F1) -> Self {
        let segment_duration = duration / 2.0;

        let out_seg = AnimationSegment::new(segment_duration, out_fn);
        let in_seg = AnimationSegment::new(segment_duration, in_fn);

        Self {
            out_seg,
            in_seg,
            _schedule: PhantomData,
        }
    }

    /// Create a new `Animation` from the given [`AnimationSegment`]s.
    pub const fn from_segments(
        out_seg: AnimationSegment<F0>,
        in_seg: AnimationSegment<F1>,
    ) -> Self {
        Self {
            out_seg,
            in_seg,
            _schedule: PhantomData,
        }
    }

    /// Get the *out* segment duration.
    #[inline]
    fn out_dur(&self) -> f32 {
        self.out_seg.duration
    }

    /// Get the *in* segment duration.
    #[inline]
    fn in_dur(&self) -> f32 {
        self.in_seg.duration
    }

    /// Get sum duration of both animation segments.
    pub const fn sum_dur(&self) -> f32 {
        self.out_seg.duration + self.in_seg.duration
    }

    /// Call the *out* [`AnimationSegment`] function.
    #[inline]
    fn animate_out<R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        rect: egui::Rect,
        normal: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R
    where
        F0: Fn(&mut egui::Ui, f32),
    {
        let layer_id = egui::LayerId::new(ui.layer_id().order, id);
        scope_content(ui, layer_id, rect, |ui| {
            (|ui| (self.out_seg.anim_fn)(ui, normal))(ui);
            add_contents(ui)
        })
    }

    /// Call the *in* [`AnimationSegment`] function.
    #[inline]
    fn animate_in<R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        rect: egui::Rect,
        normal: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R
    where
        F1: Fn(&mut egui::Ui, f32),
    {
        let layer_id = egui::LayerId::new(ui.layer_id().order, id);
        scope_content(ui, layer_id, rect, |ui| {
            (|ui| (self.in_seg.anim_fn)(ui, normal))(ui);
            add_contents(ui)
        })
    }
}

impl<F0, F1> Animate for Animation<Sequence, F0, F1>
where
    F0: Fn(&mut egui::Ui, f32),
    F1: Fn(&mut egui::Ui, f32),
{
    type RunState = SequenceRunState;

    fn run_state(&self, progress: AnimProgress) -> Self::RunState {
        if let Some(normal) = progress.elapsed_normal(self.out_dur() as f64) {
            SequenceRunState::OutSeg(1.0 - normal)
        } else if let Some(normal) = progress
            .offset(self.out_dur() as f64)
            .elapsed_normal(self.in_dur() as f64)
        {
            SequenceRunState::InSeg(normal)
        } else {
            SequenceRunState::None
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
        let rect = ui.available_rect_before_wrap();
        match self.run_state(progress) {
            SequenceRunState::OutSeg(normal) => {
                self.animate_out(ui, id.with("_out"), rect, normal, |ui| {
                    add_contents(ui, vars.start_value())
                })
            }
            SequenceRunState::InSeg(normal) => {
                mem::clear_animation_layer(ui, id.with("_out"));
                self.animate_in(ui, id.with("_in"), rect, normal, |ui| {
                    add_contents(ui, vars.current_value())
                })
            }
            SequenceRunState::None => {
                mem::clear_start_value::<T>(ui, id);
                mem::clear_start_time(ui, id);
                mem::clear_animation_layer(ui, id.with("_out"));
                mem::clear_animation_layer(ui, id.with("_in"));

                scope_content(ui, ui.layer_id(), rect, |ui| {
                    add_contents(ui, vars.current_value())
                })
            }
        }
    }
}

impl<F0, F1> Animate for Animation<Layer, F0, F1>
where
    F0: Fn(&mut egui::Ui, f32),
    F1: Fn(&mut egui::Ui, f32),
{
    type RunState = LayerRunState;

    fn run_state(&self, progress: AnimProgress) -> Self::RunState {
        match (
            progress
                .elapsed_normal(self.out_dur() as f64)
                .map(|in_norm| 1.0 - in_norm),
            progress
                // Offset progress so both animations end at the same time.
                .offset((self.out_dur() - self.in_dur()).max(0.0) as f64)
                .elapsed_normal(self.in_dur() as f64),
        ) {
            (Some(out_norm), Some(in_norm)) => LayerRunState::Running { out_norm, in_norm },
            (Some(out_norm), None) => LayerRunState::Running {
                out_norm,
                in_norm: 0.0,
            },
            (None, Some(in_norm)) => LayerRunState::Running {
                out_norm: 0.0,
                in_norm,
            },
            (None, None) => LayerRunState::None,
        }
    }

    fn animate<T: 'static + Any + Clone + Send + Sync + Default, R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        progress: AnimProgress,
        vars: AnimValues<T>,
        mut add_contents: impl FnMut(&mut egui::Ui, T) -> R,
    ) -> R {
        let rect = ui.available_rect_before_wrap();
        match self.run_state(progress) {
            LayerRunState::Running { out_norm, in_norm } => {
                let (start_value, current_value) = vars.split();
                let (out_id, in_id) = (id.with("_out"), id.with("_in"));

                self.animate_out(ui, out_id, rect, out_norm, |ui| {
                    add_contents(ui, start_value)
                });
                self.animate_in(ui, in_id, rect, in_norm, |ui| {
                    add_contents(ui, current_value)
                })
            }
            LayerRunState::None => {
                mem::clear_start_value::<T>(ui, id);
                mem::clear_start_time(ui, id);
                mem::clear_animation_layer(ui, id.with("_out"));
                mem::clear_animation_layer(ui, id.with("_in"));

                scope_content(ui, ui.layer_id(), rect, |ui| {
                    add_contents(ui, vars.current_value())
                })
            }
        }
    }
}

impl<S> Default for Animation<S> {
    fn default() -> Self {
        Self {
            out_seg: Default::default(),
            in_seg: Default::default(),
            _schedule: PhantomData,
        }
    }
}

/// Identifies animation progression and *normal* for a sequential [`Sequence`] [`Animation`].
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub enum SequenceRunState {
    /// The *out* animation segment normal.
    OutSeg(f32),
    /// The *in* animation segment normal.
    InSeg(f32),
    /// The animation is not currently running.
    #[default]
    None,
}

impl SequenceRunState {
    /// Returns `true` if either animation segment is currently running.
    pub fn is_running(&self) -> bool {
        match self {
            SequenceRunState::OutSeg(_) | SequenceRunState::InSeg(_) => true,
            SequenceRunState::None => false,
        }
    }
}

/// Identifies animation progression and *normal* for a simultaneous [`Layer`] [`Animation`].
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub enum LayerRunState {
    /// One or both segments of the animation are still running.
    Running { out_norm: f32, in_norm: f32 },
    /// The animation is not currently running.
    #[default]
    None,
}

impl LayerRunState {
    /// Returns `true` if either animation segment is currently running.
    pub fn is_running(&self) -> bool {
        match self {
            LayerRunState::Running { .. } => true,
            LayerRunState::None => false,
        }
    }
}

/// Pass the [`egui::Ui`] content to an inner scope.
pub(super) fn scope_content<R>(
    ui: &mut egui::Ui,
    layer_id: egui::LayerId,
    rect: egui::Rect,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("animation_scope")
            .max_rect(rect)
            .layer_id(layer_id),
        add_contents,
    )
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_run_state() {
        const ANIM: Animation<Sequence> = Animation::new(2.0, |_, _| {}, |_, _| {});

        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 0.0)),
            SequenceRunState::OutSeg(1.0)
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 0.5)),
            SequenceRunState::OutSeg(0.5)
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 1.0)),
            SequenceRunState::InSeg(0.0)
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 1.5)),
            SequenceRunState::InSeg(0.5)
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 2.0)),
            SequenceRunState::None
        );
    }

    #[test]
    fn short_out_layer_run_state() {
        const ANIM: Animation<Layer> = Animation::from_segments(
            AnimationSegment::new(1.0, |_, _| {}),
            AnimationSegment::new(2.0, |_, _| {}),
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 0.0)),
            LayerRunState::Running {
                out_norm: 1.0,
                in_norm: 0.0
            }
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 0.5)),
            LayerRunState::Running {
                out_norm: 0.5,
                in_norm: 0.25
            }
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 1.0)),
            LayerRunState::Running {
                out_norm: 0.0,
                in_norm: 0.5
            }
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 1.5)),
            LayerRunState::Running {
                out_norm: 0.0,
                in_norm: 0.75
            }
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 2.0)),
            LayerRunState::None
        );
    }

    #[test]
    fn short_in_layer_run_state() {
        const ANIM: Animation<Layer> = Animation::from_segments(
            AnimationSegment::new(2.0, |_, _| {}),
            AnimationSegment::new(1.0, |_, _| {}),
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 0.0)),
            LayerRunState::Running {
                out_norm: 1.0,
                in_norm: 0.0
            }
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 0.5)),
            LayerRunState::Running {
                out_norm: 0.75,
                in_norm: 0.0
            }
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 1.0)),
            LayerRunState::Running {
                out_norm: 0.5,
                in_norm: 0.0
            }
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 1.5)),
            LayerRunState::Running {
                out_norm: 0.25,
                in_norm: 0.5
            }
        );
        assert_eq!(
            ANIM.run_state(AnimProgress::new(0.0, 2.0)),
            LayerRunState::None
        );
    }
}
