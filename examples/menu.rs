use std::fmt::Display;

use eframe::NativeOptions;
use egui::emath::TSTransform;
use egui::emath::easing::{quadratic_in, quadratic_out};
use egui::{Button, Label};
use egui_animate::{Animation, animate};

/// The distance to slide out/in.
const SLIDE_DISTANCE: f32 = 10.0;
const ANIM_DURATION: f32 = 0.3;

/// The menu forward animation.
mod forward {
    use super::*;

    fn out_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_in(normal);

        ui.set_opacity(1.0 - normal);
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation((normal as f32 * -SLIDE_DISTANCE, 0.0).into()),
        );
    }
    fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_out(normal);

        ui.set_opacity(normal);
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation(
                (SLIDE_DISTANCE + normal as f32 * -SLIDE_DISTANCE, 0.0).into(),
            ),
        );
    }
    pub const ANIMATION: Animation = Animation::new(ANIM_DURATION, out_fn, in_fn);
}

/// The menu back animation.
mod back {
    use super::*;

    fn out_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_in(normal);

        ui.set_opacity(1.0 - normal);
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation((normal as f32 * SLIDE_DISTANCE, 0.0).into()),
        );
    }
    fn in_fn(ui: &mut egui::Ui, normal: f32) {
        let normal = quadratic_out(normal);

        ui.set_opacity(normal);
        ui.ctx().set_transform_layer(
            ui.layer_id(),
            TSTransform::from_translation(
                (-SLIDE_DISTANCE + normal as f32 * SLIDE_DISTANCE, 0.0).into(),
            ),
        );
    }
    pub const ANIMATION: Animation = Animation::new(ANIM_DURATION, out_fn, in_fn);
}

struct MenuApp {
    anim: Animation,
    menu_state: MenuState,
    opt1_state: OptionState,
    opt2_state: OptionState,
}

impl Default for MenuApp {
    fn default() -> Self {
        MenuApp {
            anim: forward::ANIMATION,
            menu_state: MenuState::default(),
            opt1_state: OptionState::Red,
            opt2_state: OptionState::Red,
        }
    }
}

impl eframe::App for MenuApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Menu Example");
            ui.label("This example demonstrates:");
            ui.label("• Animating an entire ui scope");
            ui.label("• Contextually setting forward/back animations");
            ui.label("• Nesting animations");
            ui.separator();

            animate(
                ui,
                "menu_anim",
                self.menu_state,
                self.anim,
                |ui, menu| match menu {
                    MenuState::MainMenu => {
                        if ui.button("New Game").clicked() {
                            self.anim = forward::ANIMATION;
                            self.menu_state = MenuState::NewGame;
                        }
                        if ui.button("Options").clicked() {
                            self.anim = forward::ANIMATION;
                            self.menu_state = MenuState::Options;
                        }
                        ui.add_enabled(false, Button::new("Quit"));
                    }
                    MenuState::NewGame => {
                        ui.add_enabled(false, Button::new("Start Game"));
                        if ui.button("Back").clicked() {
                            self.anim = back::ANIMATION;
                            self.menu_state = MenuState::MainMenu;
                        }
                    }
                    MenuState::Options => {
                        ui.horizontal(|ui| {
                            ui.label("Option 1");
                            animate(
                                ui,
                                "opt1_anim",
                                self.opt1_state,
                                forward::ANIMATION,
                                |ui, opt| {
                                    if ui.button(format!("{}", opt)).clicked() {
                                        self.opt1_state.next();
                                    }
                                },
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Option 2");
                            animate(
                                ui,
                                "opt2_anim",
                                self.opt2_state,
                                forward::ANIMATION,
                                |ui, opt| {
                                    if ui.button(format!("{}", opt)).clicked() {
                                        self.opt2_state.next();
                                    }
                                },
                            );
                        });
                        if ui.button("Back").clicked() {
                            self.anim = back::ANIMATION;
                            self.menu_state = MenuState::Confirm;
                        }
                    }
                    MenuState::Confirm => {
                        ui.label("Save changes?");
                        ui.add_enabled(false, Label::new("(This does nothing)"));

                        if ui.button("Yes").clicked() {
                            self.anim = back::ANIMATION;
                            self.menu_state = MenuState::MainMenu;
                        }
                        if ui.button("No").clicked() {
                            self.anim = back::ANIMATION;
                            self.menu_state = MenuState::MainMenu;
                        }
                    }
                },
            );
        });
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
enum MenuState {
    #[default]
    MainMenu,
    NewGame,
    Options,
    Confirm,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum OptionState {
    #[default]
    Red,
    Green,
    Blue,
}

impl Display for OptionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OptionState::Red => "Red",
                OptionState::Green => "Green",
                OptionState::Blue => "Blue",
            }
        )
    }
}

impl OptionState {
    fn next(&mut self) {
        match self {
            OptionState::Red => *self = OptionState::Green,
            OptionState::Green => *self = OptionState::Blue,
            OptionState::Blue => *self = OptionState::Red,
        }
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Menu Example",
        NativeOptions::default(),
        Box::new(|_| Ok(Box::<MenuApp>::default())),
    )
}
