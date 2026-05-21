/// The implementing type of [`AnimFn`].
pub(super) type AnimFnType = for<'a> fn(&'a mut egui::Ui, f32);

/// An animation function.
pub(super) trait AnimFn: Copy {
    /// Run the animation for a single tick.
    fn tick<'a>(self, ui: &'a mut egui::Ui, normal: f32);
}

impl<F> AnimFn for F
where
    for<'a> F: FnMut(&'a mut egui::Ui, f32) + Copy,
{
    #[inline]
    fn tick<'a>(mut self, ui: &'a mut egui::Ui, normal: f32) {
        self(ui, normal)
    }
}
