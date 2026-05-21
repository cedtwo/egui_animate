# egui_animate

## Egui Animate
Custom animations and transitions.

### Features

- *Out*/*in* `egui::Ui` element or variable transitioning.
- *Out* animations for hiding `egui::Ui` elements or variables.
- *In* animations for presenting `egui::Ui` elements or variables.
- Individual durations for *out*/*in* animation segments.
- Direct access to a scoped `&mut egui::Ui` for custom animations.

### Functionality

`egui_animate` offers simple, customizable animations based on state variables.
Define transitioning or individual *out*/*in* animations for entire ui interfaces,
and/or individual variables. Animations can be customized by providing your own
`fn(&mut egui::Ui, f32)` definitions that mutate a scoped `egui::Ui`. See the
example project for common animations/transitions.

### Animations

Animations define function(s) that mutate the `egui::Ui` given the normalized
progression of the animation (segment) between `0.0` and `1.0` (named *normal* in
examples). Calling [`animate`] will scope all `egui::Ui` mutations within. Animations
are triggered by mutations of the value passed to the `animate` function. Any variable
implementing `Default`, `PartialEq` and `Clone` is supported.

In the example below, an `egui::Label` is displayed by linearly mutating opacity
from `0.0` to `1.0` over a duration of 0.5 seconds. Animation begins when `show_ui`
is set to `true`.

```rust
# use egui;
# use eframe;
# use egui_animate::*;
// A animation that sets the (scoped) ui opacity from 0.0 to 1.0 over 0.5 seconds.
const FADE_IN: Animation = Animation::new_in(0.5, |ui, normal| ui.set_opacity(normal));

// Ui state.
let mut show_ui = false;

# let ctx = egui::Context::default();
# ctx.run(egui::RawInput::default(), |ctx| {
# egui::CentralPanel::default().show(ctx, |ui| {
if ui.button("Click to show").clicked() {
    show_ui = true;
}

animate(
    ui,                // The `egui::Ui`.
    "anim",            // A *unique* name assigned to the animation layer.
    show_ui,           // The variable that will trigger animation on change.
    FADE_IN,           // The animation (for the current frame).
    |ui, show_ui| {    // The scoped `egui::Ui` that all mutations are applied to.
        if show_ui {
            // Show our label.
            ui.label("I am in scope");
        } else {
            // Show nothing.
        }
    },
);
# });
# });
```

The following animation transitions between interfaces, dynamically setting the
animation on input.

```rust
# use egui;
# use eframe;
# use egui_animate::*;
# const SLIDE_FADE_LEFT: Animation = Animation::EMPTY;
# const SLIDE_FADE_RIGHT: Animation = Animation::EMPTY;
// Left and right slide animations. See the example project for definitions.
// const SLIDE_FADE_LEFT: Animation = ..;
// const SLIDE_FADE_RIGHT: Animation = ..;

#[derive(Default, Clone, Copy, PartialEq)]
enum MyMenu {
    #[default]
    MainMenu,
    Options,
}

// Store the menu state and the animation to dynamically change it on input.
let mut menu_state = MyMenu::MainMenu;
let mut menu_anim_state = SLIDE_FADE_LEFT;

# let ctx = egui::Context::default();
# ctx.run(egui::RawInput::default(), |ctx| {
# egui::CentralPanel::default().show(ctx, |ui| {
animate(
    ui,
    "menu_anim",
    menu_state,
    menu_anim_state,
    |ui, menu| match menu {
        // Display the `MainMenu` interface.
        MyMenu::MainMenu => {
            if ui.button("Options").clicked() {
                // Slide left on click.
                menu_anim_state = SLIDE_FADE_LEFT;
                menu_state = MyMenu::Options;
            }
        }
        // Display the `Options` interface.
        MyMenu::Options => {
            ui.button("Option 1");
            ui.button("Option 2");
            if ui.button("Back").clicked() {
                // Slide right on click.
                menu_anim_state = SLIDE_FADE_RIGHT;
                menu_state = MyMenu::MainMenu;
            }
        }
    },
);
# });
# });
```

### Animation Run State

The animation [`RunState`] provides insight into the current animation segment and
*normal*. Calling `run_state` will retrieve the state outside of the animations scoped
context. In the example below, `RunState` is used to disable an `egui::Button` for
the duration of an animation.

```rust
# use egui;
# use eframe;
# use egui_animate::*;
# const MY_ANIM: Animation = Animation::EMPTY;
# let mut my_state = 0u32;
# let ctx = egui::Context::default();
# ctx.run(egui::RawInput::default(), |ctx| {
# egui::CentralPanel::default().show(ctx, |ui| {
let button = egui::Button::new("Increment u32");

// Get the `RunState` of "my_anim".
if run_state(ui, "my_anim", MY_ANIM).is_running() {
    // Render a disabled button during animation.
    ui.add_enabled(false, button);
} else {
    // Render an enabled button when not animating.
    if ui.add(button).clicked() {
        my_state += 1;
    }
}
// Animate "my_anim".
animate(
    ui,
    "my_anim",
    my_state,
    MY_ANIM,
    |ui, show_ui| {
        // ...
    },
);
# });
# });
```

### Examples

Name | Description
---|---
`showcase` | Various example animations.
`menu` | A minimal dynamic "Main Menu" example.
`variable` | Dynamic increment/decrement animations.

```bash
cargo run --example [EXAMPLE]
```

### Compatibility

egui | egui_animate
---|---
0.32 | 0.2
