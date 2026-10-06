use eframe::NativeOptions;
use egui::emath::easing::quadratic_out;
use egui::{Button, RichText};
use egui_animate::prelude::*;

const ANIM_DURATION: f32 = 0.6;

/*
    Animation and animation function definitions.
*/

const FORWARD_ANIM: Animation<Layer> = Animation::new_layer(ANIM_DURATION, anim_left, anim_right);
const BACKWARD_ANIM: Animation<Layer> = Animation::new_layer(ANIM_DURATION, anim_right, anim_left);

fn anim_left(ui: &mut egui::Ui, normal: f32) {
    let normal = quadratic_out(normal);
    scale(ui, normal, ui.max_rect().center(), 1.1);
    fade(ui, normal);
}

fn anim_right(ui: &mut egui::Ui, normal: f32) {
    let normal = quadratic_out(normal);
    scale(ui, normal, ui.max_rect().center(), 0.9);
    fade(ui, normal);
}

/*
    App, animation state and ui helper functions.
*/

#[derive(Default, Debug, Clone, Copy)]
struct LayerApp {
    anim: Animation<Layer>,
    page: Page,
}

impl LayerApp {
    fn render_back_button(&mut self, ui: &mut egui::Ui, page: Page) {
        if let Some((prev, anim)) = page.prev() {
            if ui.button("Back").clicked() {
                self.page = prev;
                self.anim = anim;
            }
        } else {
            ui.add_enabled(false, Button::new("Back"));
        }
    }

    fn render_next_button(&mut self, ui: &mut egui::Ui, page: Page) {
        if let Some((next, anim)) = page.next() {
            if ui.button("Next").clicked() {
                self.page = next;
                self.anim = anim;
            }
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Page {
    #[default]
    Title,
    Page0,
    Page1,
    Page2,
}

impl Page {
    fn render_heading(&self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading(match self {
            Page::Title => "Title",
            Page::Page0 => "Page 0",
            Page::Page1 => "Page 1",
            Page::Page2 => "Page 2",
        })
    }
}

impl Page {
    fn next(self) -> Option<(Self, Animation<Layer>)> {
        match self {
            Page::Title => Some((Page::Page0, FORWARD_ANIM)),
            Page::Page0 => Some((Page::Page1, FORWARD_ANIM)),
            Page::Page1 => Some((Page::Page2, FORWARD_ANIM)),
            Page::Page2 => None,
        }
    }

    fn prev(self) -> Option<(Self, Animation<Layer>)> {
        match self {
            Page::Title => None,
            Page::Page0 => Some((Page::Title, BACKWARD_ANIM)),
            Page::Page1 => Some((Page::Page0, BACKWARD_ANIM)),
            Page::Page2 => Some((Page::Page1, BACKWARD_ANIM)),
        }
    }
}

/*
    Egui logic.
*/

impl eframe::App for LayerApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            animate(ui, "menu_anim", self.page, self.anim, |ui, page| {
                page.render_heading(ui);
                ui.label(RichText::new("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Laborum veritatis quia eiusmod nostrud fugiat. Iusto nemo in nisi nisi et beatae ullamco irure odit anim mollit deserunt exercitation. Quasi ab velit enim adipiscing modi laboris cupiditate vitae nisi anim commodo animi. Irure porro magni quis officia excepturi adipiscing aspernatur magni. Occaecat praesentium anim eiusmod nesciunt."));
                ui.horizontal(|ui| {
                    self.render_back_button(ui, page);
                    self.render_next_button(ui, page);
                })
            });
            let run_state = match run_state(ui, "menu_anim", self.anim).is_running() {
                true => "Running...",
                false => "Stopped.",
            };
            ui.separator();
            ui.label(format!("Animation State: {}", run_state));
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Layer Animation Example",
        NativeOptions::default(),
        Box::new(|_| Ok(Box::<LayerApp>::default())),
    )
}
