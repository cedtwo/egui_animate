use std::any::Any;

use crate::state::{AnimProgress, AnimValues};

/// Trait for running animation logic and retrieving an animation progression.
pub trait Animate {
    /// The current progression of an animation.
    type RunState;

    /// Get the [`RunState`](Self::RunState) for the current frame.
    fn run_state(&self, progress: AnimProgress) -> Self::RunState;

    /// Update the animation for the current tick.
    fn animate<T: 'static + Any + Clone + Send + Sync + Default, R>(
        &self,
        ui: &mut egui::Ui,
        id: egui::Id,
        progress: AnimProgress,
        vars: AnimValues<T>,
        add_contents: impl FnOnce(&mut egui::Ui, T) -> R,
    ) -> R;
}
