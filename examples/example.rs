use eframe::NativeOptions;
use egui::emath::TSTransform;
use egui::emath::easing::{quadratic_in, quadratic_out};
use egui::{InnerResponse, RichText};
use egui_animate::{Animation, AnimationSegment, animate};

/// The distance to slide out/in.
const SLIDE_DISTANCE: f32 = 10.0;

/*
    # Example Animations

    Below are some common and more abstract animations that can be used to transition
    between variables. An animation can be constructed with:
    ```
    const ANIM: Animation = Animation::new(duration, out_fn, in_fn);
    ```
*/

mod fade {
    pub fn out_fn(ui: &mut egui::Ui, normal: f32) {
        ui.set_opacity(1.0 - normal);
    }
    pub fn in_fn(ui: &mut egui::Ui, normal: f32) {
        ui.set_opacity(normal);
    }
}

mod slide_left {
    use super::*;

    pub fn out_fn(ui: &mut egui::Ui, normal: f32) {
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation((normal as f32 * -SLIDE_DISTANCE, 0.0).into()),
        );
    }
    pub fn in_fn(ui: &mut egui::Ui, normal: f32) {
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation(
                (SLIDE_DISTANCE + normal as f32 * -SLIDE_DISTANCE, 0.0).into(),
            ),
        );
    }
}

mod slide_right {
    use super::*;

    pub fn out_fn(ui: &mut egui::Ui, normal: f32) {
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation((normal as f32 * SLIDE_DISTANCE, 0.0).into()),
        );
    }
    pub fn in_fn(ui: &mut egui::Ui, normal: f32) {
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation(
                (-SLIDE_DISTANCE + normal as f32 * SLIDE_DISTANCE, 0.0).into(),
            ),
        );
    }
}

mod slide_fade_ease_left {
    use super::*;

    pub fn out_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_in(normal);

        fade::out_fn(ui, normal);
        slide_left::out_fn(ui, normal);
    }
    pub fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_out(normal);

        fade::in_fn(ui, normal);
        slide_left::in_fn(ui, normal);
    }
}

mod slide_fade_ease_right {
    use super::*;

    pub fn out_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_in(normal);
        fade::out_fn(ui, normal);
        slide_right::out_fn(ui, normal);
    }
    pub fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_out(normal);
        fade::in_fn(ui, normal);
        slide_right::in_fn(ui, normal);
    }
}

mod clip_width {
    pub fn out_fn(ui: &mut egui::Ui, normal: f32) {
        in_fn(ui, 1.0 - normal);
    }
    pub fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let mut rect = ui.clip_rect();
        rect.set_width(rect.width() * normal);
        ui.set_clip_rect(rect);
    }
}

mod clip_height {
    pub fn out_fn(ui: &mut egui::Ui, normal: f32) {
        in_fn(ui, 1.0 - normal);
    }
    pub fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let mut rect = ui.clip_rect();
        rect.set_height(rect.height() * normal);
        ui.set_clip_rect(rect);
    }
}

mod fade_green {
    use super::*;

    pub fn out_fn(ui: &mut egui::Ui, normal: f32) {
        in_fn(ui, 1.0 - normal);
    }
    pub fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let inverse_normal = 1.0 - normal as f32;

        let mut text_color = ui.visuals_mut().text_color();
        let red_color_range = (255 - text_color[1]) as f32;
        text_color[1] += (red_color_range * inverse_normal).min(255.0) as u8;
        ui.visuals_mut().override_text_color = Some(text_color);
        ui.set_opacity(normal);

        fade::in_fn(ui, normal);
    }
}

mod fade_red {
    use super::*;

    pub fn out_fn(ui: &mut egui::Ui, normal: f32) {
        in_fn(ui, 1.0 - normal);
    }
    pub fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let inverse_normal = 1.0 - normal as f32;

        let mut text_color = ui.visuals_mut().text_color();
        let red_color_range = (255 - text_color[0]) as f32;
        text_color[0] += (red_color_range * inverse_normal).min(255.0) as u8;
        ui.visuals_mut().override_text_color = Some(text_color);
        ui.set_opacity(normal);

        fade::in_fn(ui, normal);
    }
}

/**
    # Animation Type

    Used for constructing an `Animation` from the configured variant, mapping to
    the out/in functions. Simplifies construction of an animation for `ExampleApp`.
    Typically defining a `const` Animation is sufficient for most use cases.
*/
#[repr(usize)]
#[derive(Default, PartialEq, Clone, Copy)]
enum AnimationType {
    Fade,
    SlideFadeEaseLeft,
    #[default]
    SlideFadeEaseRight,
    ClipWidth,
    ClipHeight,
    FadeRed,
    FadeGreen,
}

impl AnimationType {
    fn label(&self) -> String {
        match self {
            AnimationType::Fade => "Fade",
            AnimationType::SlideFadeEaseLeft => "Slide fade ease left",
            AnimationType::SlideFadeEaseRight => "Slide fade ease right",
            AnimationType::ClipWidth => "Clip width",
            AnimationType::ClipHeight => "Clip height",
            AnimationType::FadeRed => "Fade red",
            AnimationType::FadeGreen => "Fade green",
        }
        .to_string()
    }

    fn combo_box(&mut self, ui: &mut egui::Ui, label: &str) -> InnerResponse<Option<()>> {
        egui::ComboBox::from_label(label)
            .selected_text(self.label())
            .show_ui(ui, |ui| {
                self.selectable_value(ui, AnimationType::Fade);
                self.selectable_value(ui, AnimationType::SlideFadeEaseLeft);
                self.selectable_value(ui, AnimationType::SlideFadeEaseRight);
                self.selectable_value(ui, AnimationType::ClipWidth);
                self.selectable_value(ui, AnimationType::ClipHeight);
                self.selectable_value(ui, AnimationType::FadeRed);
                self.selectable_value(ui, AnimationType::FadeGreen);
            })
    }

    fn selectable_value(&mut self, ui: &mut egui::Ui, value: Self) -> egui::Response {
        let label = value.label();
        ui.selectable_value(self, value, label)
    }

    pub fn out_fn(&self) -> fn(&mut egui::Ui, f32) {
        match self {
            AnimationType::Fade => fade::out_fn,
            AnimationType::SlideFadeEaseLeft => slide_fade_ease_left::out_fn,
            AnimationType::SlideFadeEaseRight => slide_fade_ease_right::out_fn,
            AnimationType::ClipWidth => clip_width::out_fn,
            AnimationType::ClipHeight => clip_height::out_fn,
            AnimationType::FadeRed => fade_red::out_fn,
            AnimationType::FadeGreen => fade_green::out_fn,
        }
    }

    fn in_fn(&self) -> fn(&mut egui::Ui, f32) {
        match self {
            AnimationType::Fade => fade::in_fn,
            AnimationType::SlideFadeEaseLeft => slide_fade_ease_left::in_fn,
            AnimationType::SlideFadeEaseRight => slide_fade_ease_right::in_fn,
            AnimationType::ClipWidth => clip_width::in_fn,
            AnimationType::ClipHeight => clip_height::in_fn,
            AnimationType::FadeRed => fade_red::in_fn,
            AnimationType::FadeGreen => fade_green::in_fn,
        }
    }
}

/**
    # Example App

    Creates an `Animation` from a given configuration. Stores the state of the
    animated value.
*/
struct ExampleApp {
    /// The value to animate on change.
    value_state: u8,

    // Out animation configuration.
    out_anim: AnimationType,
    out_dur: f32,

    // In animation configuration.
    in_copy_from_out: bool,
    in_anim: AnimationType,
    in_dur: f32,
}

impl Default for ExampleApp {
    fn default() -> Self {
        ExampleApp {
            value_state: 0,
            out_dur: 0.4,
            in_dur: 0.4,
            out_anim: AnimationType::default(),
            in_anim: AnimationType::default(),
            in_copy_from_out: true,
        }
    }
}

impl ExampleApp {
    /// Create an `Animation` from given configuration.
    fn into_anim(&self) -> Animation {
        let out_seg = AnimationSegment {
            duration: self.out_dur,
            anim_fn: self.out_anim.out_fn(),
        };
        let in_seg = AnimationSegment {
            duration: self.in_dur,
            anim_fn: self.in_anim.in_fn(),
        };
        Animation::from_segments(out_seg, in_seg)
    }
}

impl eframe::App for ExampleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Animation config");
            ui.separator();
            ui.label("A collection of custom example animations");

            ui.group(|ui| {
                ui.label("Animation for the prior value before transition");
                ui.add(egui::Slider::new(&mut self.out_dur, 0.0..=2.0).text("Duration"));
                self.out_anim.combo_box(ui, "Out animation type");
            });

            ui.group(|ui| {
                ui.label("Animation for the next value after transition");
                ui.checkbox(&mut self.in_copy_from_out, "Copy from 'out' configuration");

                if self.in_copy_from_out {
                    self.in_dur = self.out_dur;
                    self.in_anim = self.out_anim;

                    ui.disable();
                }
                ui.add(egui::Slider::new(&mut self.in_dur, 0.0..=2.0).text("Duration"));
                self.in_anim.combo_box(ui, "In animation type");
            });

            ui.heading("Example");
            ui.separator();

            animate(
                ui,
                "int_anim",
                self.value_state,
                self.into_anim(),
                |ui, value| {
                    let text = RichText::new(format!("Int: {}", value)).size(48.0);
                    ui.label(text);
                    ui.label(format!(
                        "Animation: {} / {}",
                        self.out_anim.label(),
                        self.in_anim.label()
                    ));
                    ui.label(format!("Total duration: {}", self.out_dur + self.in_dur));

                    ui.horizontal(|ui| {
                        if ui.button("Decrement").clicked() {
                            self.value_state = value.checked_sub(1).unwrap_or(0);
                        };
                        if ui.button("Increment").clicked() {
                            self.value_state = value.checked_add(1).unwrap_or(u8::MAX);
                        };
                    });
                },
            );
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Minimal Example",
        NativeOptions::default(),
        Box::new(|_| Ok(Box::<ExampleApp>::default())),
    )
}
