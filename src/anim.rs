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
#[derive(Default, Clone, Copy)]
pub struct Animation {
    /// The segment animating the prior value **out**.
    pub out_seg: AnimationSegment,
    /// The segment animating the new value **in**.
    pub in_seg: AnimationSegment,
}

impl Animation {
    /// An empty placeholder animation.
    pub const EMPTY: Animation =
        Animation::from_segments(AnimationSegment::EMPTY, AnimationSegment::EMPTY);

    /// Create a new `Animation` with the given total `duration`, split over segments.
    pub const fn new(
        duration: f32,
        out_fn: fn(&mut egui::Ui, f32),
        in_fn: fn(&mut egui::Ui, f32),
    ) -> Self {
        let segment_duration = duration / 2.0;

        let out_seg = AnimationSegment::new(segment_duration, out_fn);
        let in_seg = AnimationSegment::new(segment_duration, in_fn);

        Self { out_seg, in_seg }
    }

    /// Create a new `Animation` with only the *out* segment. Passes the the prior
    /// value to the animation scope for the duration of the `out_fn`.
    pub const fn new_out(duration: f32, out_fn: fn(&mut egui::Ui, f32)) -> Self {
        let out_seg = AnimationSegment::new(duration, out_fn);
        let in_seg = AnimationSegment::EMPTY;

        Self { out_seg, in_seg }
    }

    /// Create a new `Animation` with only the *in* segment. Passes the the mutated
    /// value to the animation scope for the duration of the `in_fn`.
    pub const fn new_in(duration: f32, out_fn: fn(&mut egui::Ui, f32)) -> Self {
        let out_seg = AnimationSegment::EMPTY;
        let in_seg = AnimationSegment::new(duration, out_fn);

        Self { out_seg, in_seg }
    }

    /// Create a new `Animation` from the given [`AnimationSegment`]s.
    pub const fn from_segments(out_seg: AnimationSegment, in_seg: AnimationSegment) -> Self {
        Self { out_seg, in_seg }
    }

    /// Get the total duration of the animation.
    pub const fn duration(&self) -> f32 {
        self.out_seg.duration + self.in_seg.duration
    }
}

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
pub struct AnimationSegment {
    /// The duration of the animation, in seconds.
    pub duration: f32,
    /// The [`Ui`] mutating function for the given `f32` normal.
    pub anim_fn: fn(&mut egui::Ui, f32),
}

impl Default for AnimationSegment {
    fn default() -> Self {
        AnimationSegment::EMPTY
    }
}

impl AnimationSegment {
    /// An empty placeholder animation segment.
    const EMPTY: AnimationSegment = AnimationSegment {
        duration: 0.0,
        anim_fn: |_, _| {},
    };

    /// Create a new `AnimationSegment` from the given `duration` and `animation` function.
    pub const fn new(duration: f32, animation: fn(&mut egui::Ui, f32)) -> Self {
        Self {
            duration,
            anim_fn: animation,
        }
    }

    /// Get the animation duration.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn duration_mut(&mut self) -> f32 {
        self.duration
    }

    /// Get the animation function.
    pub fn anim_fn(&self) -> fn(&mut egui::Ui, f32) {
        self.anim_fn
    }

    /// Get a mutable reference to the animation function.
    pub fn anim_fn_mut(&mut self) -> &mut fn(&mut egui::Ui, f32) {
        &mut self.anim_fn
    }

    /// Apply the animation function, passing in the given `normal`.
    pub(super) fn animate<R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        normal: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        Self::scope_animation(ui, id, |ui| (self.anim_fn)(ui, normal), add_contents)
    }

    /// Create a child [`egui::Ui`] for animation.
    fn scope_animation<R>(
        ui: &mut egui::Ui,
        id: egui::Id,
        anim_fn: impl FnOnce(&mut egui::Ui),
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let layer_id = Self::animation_layer(ui, id);
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

    /// Get the animation layer id.
    pub(crate) fn animation_layer(ui: &mut egui::Ui, id: egui::Id) -> egui::LayerId {
        egui::LayerId::new(ui.layer_id().order, id)
    }
}
