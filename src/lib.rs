//! # Egui Animate
//! Custom animations and transitions.
//!
//! ## Features
//!
//! - *Out*/*in* `egui::Ui` element or variable transitioning.
//! - *Out* animations for hiding `egui::Ui` elements or variables.
//! - *In* animations for presenting `egui::Ui` elements or variables.
//! - Individual durations for *out*/*in* animation segments.
//! - Direct access to a scoped `&mut egui::Ui` for custom animations.
//!
//! ## Functionality
//!
//! `egui_animate` offers simple, customizable animations based on state variables.
//! Define transitioning or individual *out*/*in* animations for entire ui interfaces,
//! and/or individual variables. Animations can be customized by providing your own
//! `fn(&mut egui::Ui, f32)` definitions that mutate a scoped `egui::Ui`. See the
//! example project for common animations/transitions.
//!
//! ## Animations
//!
//! Animations define function(s) that mutate the `egui::Ui` given the normalized
//! progression of the animation (segment) between `0.0` and `1.0` (named *normal* in
//! examples). Calling [`animate`] will scope all `egui::Ui` mutations within. Animations
//! are triggered by mutations of the value passed to the `animate` function. Any variable
//! implementing `Default`, `PartialEq` and `Clone` is supported.
//!
//! In the example below, an `egui::Label` is displayed by linearly mutating opacity
//! from `0.0` to `1.0` over a duration of 0.5 seconds. Animation begins when `show_ui`
//! is set to `true`.
//!
//! ```
//! # use egui;
//! # use eframe;
//! # use egui_animate::*;
//! // Animation definition
//! const FADE_IN: Animation = Animation::new_in(0.5, |ui, normal| ui.set_opacity(normal));
//!
//! // Ui state.
//! let mut show_ui = false;
//!
//! # let ctx = egui::Context::default();
//! # ctx.run(egui::RawInput::default(), |ctx| {
//! # egui::CentralPanel::default().show(ctx, |ui| {
//! if ui.button("Click to show").clicked() {
//!     show_ui = true;
//! }
//!
//! animate(
//!     ui,                // The `egui::Ui`.
//!     "anim",            // A *unique* name assigned to the animation layer.
//!     show_ui,           // The variable that will trigger animation on change.
//!     FADE_IN,           // The animation (for the current frame).
//!     |ui, show_ui| {    // The scoped `egui::Ui` that all mutations are applied to.
//!         if show_ui {
//!             // Show our label.
//!             ui.label("I am in scope");
//!         } else {
//!             // Show nothing.
//!         }
//!     },
//! );
//! # });
//! # });
//! ```
//!
//! The following animation transitions between interfaces, dynamically setting the
//! animation on input.
//!
//! ```
//! # use egui;
//! # use eframe;
//! # use egui_animate::*;
//! # const SLIDE_FADE_LEFT: Animation = Animation::EMPTY;
//! # const SLIDE_FADE_RIGHT: Animation = Animation::EMPTY;
//! // Left and right slide animations. See the example project for definitions.
//! // const SLIDE_FADE_LEFT: Animation = ..;
//! // const SLIDE_FADE_RIGHT: Animation = ..;
//!
//! #[derive(Default, Clone, Copy, PartialEq)]
//! enum MyMenu {
//!     #[default]
//!     MainMenu,
//!     Options,
//! }
//!
//! // Store the menu state and the animation to dynamically change it on input.
//! let mut menu_state = MyMenu::MainMenu;
//! let mut menu_anim_state = SLIDE_FADE_LEFT;
//!
//! # let ctx = egui::Context::default();
//! # ctx.run(egui::RawInput::default(), |ctx| {
//! # egui::CentralPanel::default().show(ctx, |ui| {
//! animate(
//!     ui,
//!     "menu_anim",
//!     menu_state,
//!     menu_anim_state,
//!     |ui, menu| match menu {
//!         // Display the `MainMenu` interface.
//!         MyMenu::MainMenu => {
//!             if ui.button("Options").clicked() {
//!                 // Slide left on click.
//!                 menu_anim_state = SLIDE_FADE_LEFT;
//!                 menu_state = MyMenu::Options;
//!             }
//!         }
//!         // Display the `Options` interface.
//!         MyMenu::Options => {
//!             ui.button("Option 1");
//!             ui.button("Option 2");
//!             if ui.button("Back").clicked() {
//!                 // Slide right on click.
//!                 menu_anim_state = SLIDE_FADE_RIGHT;
//!                 menu_state = MyMenu::MainMenu;
//!             }
//!         }
//!     },
//! );
//! # });
//! # });
//! ```
//!
//! ## Examples
//!
//! An example app is provided that demonstrates various transitions including fading,
//! sliding, clipping, mutating text color, and easing.
//!
//! ```bash
//! cargo run --example example
//! ```
mod mem;

mod anim;
mod state;

pub use anim::{Animation, AnimationSegment};
pub use state::animate;
