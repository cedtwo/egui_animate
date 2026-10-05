use std::any::Any;
use std::marker::PhantomData;

use crate::animate::Animate;
use crate::mem;
use crate::schedule::{Layer, Sequence};
use crate::segment::{AnimFn, AnimationSegment};
use crate::state::{AnimProgress, AnimValues};

/// An animation defined by [`AnimationSegment`]\(s).
///
/// An animation must include either an *out* function, an *in* function, or both. Single function
/// animations may be suitable for displaying or hiding elements, while out/in animations simplify
/// transitions.
///
/// # Animation Functions
///
/// Animation functions recieve a *normal*, representing the linear relative progress (`0.0` to
/// `1.0`) of an animation segment. This can be passed to easing functions for smoother animations,
/// and often reversed to reverse a previously defined animation.
///
/// ## Example
/// ```
/// # use egui_animate::{Animation, Sequence};
/// // A 0.2 second fade out/in animation.
/// const ANIM: Animation<Sequence> = Animation::new(
///     0.2,
///     |ui, normal| ui.set_opacity(1.0 - normal),
///     |ui, normal| ui.set_opacity(normal),
/// );
/// ```
///
/// # Animation Scheduling
///
/// `Animation` supports running each segment sequentially or at the same time by passing either
/// [`Sequence`] or [`Layer`] as the first generic argument `S`. See the respective documentation of
/// each marker type for more.
#[derive(Clone, Copy)]
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
}

impl<S, F0, F1> Animation<S, F0, F1> {
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
        self.out_seg
            .animate_scoped(ui, id, rect, normal, add_contents)
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
        self.in_seg
            .animate_scoped(ui, id, rect, normal, add_contents)
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
        match self.run_state(progress) {
            SequenceRunState::OutSeg(normal) => self.animate_out(
                ui,
                id.with("_out"),
                ui.available_rect_before_wrap(),
                normal,
                |ui| add_contents(ui, vars.start_value()),
            ),
            SequenceRunState::InSeg(normal) => {
                mem::clear_animation_layer(ui, id);
                self.animate_in(
                    ui,
                    id.with("_in"),
                    ui.available_rect_before_wrap(),
                    normal,
                    |ui| add_contents(ui, vars.current_value()),
                )
            }
            SequenceRunState::None => {
                mem::clear_start_value::<T>(ui, id);
                mem::clear_start_time(ui, id);
                mem::clear_animation_layer(ui, id);

                add_contents(ui, vars.current_value())
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
        match self.run_state(progress) {
            LayerRunState::Running { out_norm, in_norm } => {
                let (start_value, current_value) = vars.split();
                let (out_id, in_id) = (id.with("_out"), id.with("_in"));
                let rect = ui.available_rect_before_wrap();

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
                mem::clear_animation_layer(ui, id);

                add_contents(ui, vars.current_value())
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

#[cfg(test)]
mod tests {
    use super::*;

    const ANIM_SEQ: Animation<Sequence> = Animation::new(2.0, |_, _| {}, |_, _| {});

    #[test]
    fn sequence_run_state() {
        assert_eq!(
            ANIM_SEQ.run_state(AnimProgress::new(0.0, 0.0)),
            SequenceRunState::OutSeg(1.0)
        );
        assert_eq!(
            ANIM_SEQ.run_state(AnimProgress::new(0.0, 0.5)),
            SequenceRunState::OutSeg(0.5)
        );
        assert_eq!(
            ANIM_SEQ.run_state(AnimProgress::new(0.0, 1.0)),
            SequenceRunState::InSeg(0.0)
        );
        assert_eq!(
            ANIM_SEQ.run_state(AnimProgress::new(0.0, 1.5)),
            SequenceRunState::InSeg(0.5)
        );
        assert_eq!(
            ANIM_SEQ.run_state(AnimProgress::new(0.0, 2.0)),
            SequenceRunState::None
        );
    }

    const ANIM_LAY: Animation<Layer> = Animation::from_segments(
        AnimationSegment::new(2.0, |_, _| {}),
        AnimationSegment::new(1.0, |_, _| {}),
    );

    #[test]
    fn layer_run_state() {
        assert_eq!(
            ANIM_LAY.run_state(AnimProgress::new(0.0, 0.0)),
            LayerRunState::Running {
                out_norm: 1.0,
                in_norm: 0.0
            }
        );
        assert_eq!(
            ANIM_LAY.run_state(AnimProgress::new(0.0, 0.5)),
            LayerRunState::Running {
                out_norm: 0.75,
                in_norm: 0.0
            }
        );
        assert_eq!(
            ANIM_LAY.run_state(AnimProgress::new(0.0, 1.0)),
            LayerRunState::Running {
                out_norm: 0.5,
                in_norm: 0.0
            }
        );
        assert_eq!(
            ANIM_LAY.run_state(AnimProgress::new(0.0, 1.5)),
            LayerRunState::Running {
                out_norm: 0.25,
                in_norm: 0.5
            }
        );
        assert_eq!(
            ANIM_LAY.run_state(AnimProgress::new(0.0, 2.0)),
            LayerRunState::None
        );
    }
}
