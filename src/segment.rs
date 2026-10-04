/// The animation function pointer type.
pub(super) type AnimFn = fn(&mut egui::Ui, f32);

/// A single segment of the animation.
///
/// Defines the `duration` of a segment (in seconds), and a mutating function for the
/// [`Ui`](egui::Ui). See [`Animation`](crate::Animation) for details on how to construct an
/// animation function.
///
/// # Example
/// ```
/// # use egui_animate::*;
/// // A simple animation that fades the prior value out, and the new value in.
/// // Has a total time of `0.4` seconds with each segment taking a respective `0.2` seconds each.
/// const FADE_OUT: AnimationSegment = AnimationSegment::new(0.2, |ui, normal| ui.set_opacity(1.0 - normal));
/// const FADE_IN: AnimationSegment = AnimationSegment::new(0.2, |ui, normal| ui.set_opacity(normal));
///
/// const ANIM: Animation<Sequence> = Animation::from_segments(FADE_OUT, FADE_IN);
/// ```
#[derive(Clone, Copy)]
pub struct AnimationSegment<F = AnimFn> {
    /// The duration of the animation, in seconds.
    pub duration: f32,
    /// The [`Ui`](egui::Ui) mutating function for the given `f32` normal.
    pub anim_fn: F,
}

impl AnimationSegment<AnimFn> {
    /// An empty animation segment.
    pub(super) const EMPTY: Self = AnimationSegment {
        duration: 0.0,
        anim_fn: |_, _| {},
    };
}

impl<F> AnimationSegment<F> {
    /// Create a new `AnimationSegment` from the given `duration` and animation function.
    pub const fn new(duration: f32, anim_fn: F) -> Self {
        Self { duration, anim_fn }
    }

    /// Get the animation duration.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Get a new animation duration.
    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
    }
}

impl<F> AnimationSegment<F>
where
    F: Fn(&mut egui::Ui, f32),
{
    /// Call the animation function in a child [`egui::Ui`] animation scope.
    pub(super) fn animate_scoped<R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        rect: egui::Rect,
        normal: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let layer_id = egui::LayerId::new(ui.layer_id().order, id);
        ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt("animation_scope")
                .max_rect(rect)
                .layer_id(layer_id),
            |ui| {
                (|ui| (self.anim_fn)(ui, normal))(ui);
                add_contents(ui)
            },
        )
        .inner
    }
}

impl Default for AnimationSegment<AnimFn> {
    fn default() -> Self {
        Self::EMPTY
    }
}
