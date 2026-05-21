use eframe::NativeOptions;
use egui::emath::TSTransform;
use egui::emath::easing::{quadratic_in, quadratic_out};
use egui::{Button, RichText};
use egui_animate::{Animation, RunState, animate, run_state};

/// The distance to slide out/in.
const SLIDE_DISTANCE: f32 = 10.0;
const ANIM_DURATION: f32 = 0.4;

/// The variable increment animation.
mod increment {
    use super::*;

    fn out_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_in(normal);
        let inverse_normal = 1.0 - normal as f32;

        let mut text_color = ui.visuals_mut().text_color();
        text_color[1] = 255;
        ui.visuals_mut().override_text_color = Some(text_color);

        ui.set_opacity(inverse_normal);
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation((0.0, normal as f32 * -SLIDE_DISTANCE).into()),
        );
    }
    fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_out(normal);

        ui.set_opacity(normal);
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation(
                (0.0, SLIDE_DISTANCE + normal as f32 * -SLIDE_DISTANCE).into(),
            ),
        );
    }
    pub const ANIMATION: Animation = Animation::new(ANIM_DURATION, out_fn, in_fn);
}

/// The variable decrement animation.
mod decrement {
    use super::*;

    fn out_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_in(normal);
        let inverse_normal = 1.0 - normal as f32;

        let mut text_color = ui.visuals_mut().text_color();
        text_color[0] = 255;
        ui.visuals_mut().override_text_color = Some(text_color);

        ui.set_opacity(inverse_normal);
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation((0.0, normal as f32 * SLIDE_DISTANCE).into()),
        );
    }
    fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_out(normal);
        ui.set_opacity(normal);

        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation(
                (0.0, -SLIDE_DISTANCE + normal as f32 * SLIDE_DISTANCE).into(),
            ),
        );
    }
    pub const ANIMATION: Animation = Animation::new(ANIM_DURATION, out_fn, in_fn);
}

struct VariableApp {
    anim: Animation,
    state: u8,
}

impl Default for VariableApp {
    fn default() -> Self {
        VariableApp {
            anim: increment::ANIMATION,
            state: 0,
        }
    }
}

impl eframe::App for VariableApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Variable Example");
            ui.label("This example demonstrates:");
            ui.label("• Animating a single ui element");
            ui.label("• Contextually setting increment/decrement animations");
            ui.label("• Using the 'RunState' to disable buttons during the 'in' animation");
            ui.separator();

            ui.horizontal(|ui| {
                let decr_button = Button::new("-");
                let incr_button = Button::new("+");

                if let RunState::InSeg(_) = run_state(ui, "int_anim", self.anim) {
                    ui.add_enabled(false, decr_button);
                    ui.add_enabled(false, incr_button);
                } else {
                    match self.state {
                        0 => {
                            ui.add_enabled(false, decr_button);
                            if ui.add(incr_button).clicked() {
                                self.state += 1;
                                self.anim = increment::ANIMATION;
                            }
                        }
                        u8::MAX => {
                            if ui.add(decr_button).clicked() {
                                self.state -= 1;
                                self.anim = decrement::ANIMATION;
                            }
                            ui.add_enabled(false, incr_button);
                        }
                        _ => {
                            if ui.add(decr_button).clicked() {
                                self.state -= 1;
                                self.anim = decrement::ANIMATION;
                            }
                            if ui.add(incr_button).clicked() {
                                self.state += 1;
                                self.anim = increment::ANIMATION;
                            }
                        }
                    }
                }
            });

            animate(ui, "int_anim", self.state, self.anim, |ui, value| {
                let text = RichText::new(format!("{}", value)).size(48.0);
                ui.label(text);
            });
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Variable Example",
        NativeOptions::default(),
        Box::new(|_| Ok(Box::<VariableApp>::default())),
    )
}
