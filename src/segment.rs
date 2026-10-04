use crate::ty::{AnimFn, AnimPointer};

/// A single segment of the animation.
///
/// Defines the `duration` of a segment (in seconds), and a mutating function for
/// the [`Ui`]. See [`Animation`] for details on how to construct an animation function.
///
/// # Example
/// ```
/// # use egui_animate::{Animation, AnimationSegment};
/// // A simple animation that fades the prior value out, and the new value in.
/// // Has a total time of `0.4` seconds with each segment taking a respective `0.2` seconds each.
/// const FADE_OUT: AnimationSegment = AnimationSegment::new(0.2, |ui, normal| ui.set_opacity(1.0 - normal));
/// const FADE_IN: AnimationSegment = AnimationSegment::new(0.2, |ui, normal| ui.set_opacity(normal));
///
/// const ANIM: Animation = Animation::from_segments(FADE_OUT, FADE_IN);
/// ```
#[derive(Clone, Copy)]
pub struct AnimationSegment<F = AnimPointer> {
    /// The duration of the animation, in seconds.
    pub duration: f32,
    /// The [`Ui`] mutating function for the given `f32` normal.
    pub anim_fn: F,
}

impl AnimationSegment<AnimPointer> {
    /// An empty placeholder animation segment.
    pub(super) const EMPTY: Self = AnimationSegment {
        duration: 0.0,
        anim_fn: |_, _| {},
    };
}

impl<F> AnimationSegment<F> {
    /// Create a new `AnimationSegment` from the given `duration` and `animation` function.
    pub const fn new(duration: f32, anim_fn: F) -> Self {
        Self { duration, anim_fn }
    }

    /// Get the animation duration.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn duration_mut(&mut self) -> f32 {
        self.duration
    }
}

impl<F: AnimFn> AnimationSegment<F> {
    /// Get the animation function.
    pub fn anim_fn(&self) -> F {
        self.anim_fn
    }

    /// Create a child [`egui::Ui`] for animation.
    fn scope_animation<R>(
        ui: &mut egui::Ui,
        id: egui::Id,
        anim_fn: impl FnOnce(&mut egui::Ui),
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let layer_id = egui::LayerId::new(ui.layer_id().order, id);
        ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt("animation_scope")
                .layer_id(layer_id),
            |ui| {
                anim_fn(ui);
                add_contents(ui)
            },
        )
        .inner
    }

    /// Apply the animation function, passing in the given `normal`.
    pub(super) fn animate<R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        normal: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        Self::scope_animation(ui, id, |ui| self.anim_fn.tick(ui, normal), add_contents)
    }
}

impl Default for AnimationSegment<AnimPointer> {
    fn default() -> Self {
        Self::EMPTY
    }
}
