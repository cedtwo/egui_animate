# Egui Animate

Custom `egui` animations and transitions.

## Features

- Hide or reveal `egui::Ui` elements,
- Transition between elements by sequentially animating the prior state out and the new state in.
- Transition between elements by simultaneously animating the prior value out and the new value in.
- Support different durations for individual out/in animation segments.
- Direct access to a scoped `&mut egui::Ui` for custom animations.

## Functionality

`egui_animate` offers simple, customizable animations based on state variables. Define
transitioning or individual *out*/*in* animations for entire ui interfaces, and/or individual ui
elements. Animations can be customized by providing your own `FnMut(&mut egui::Ui, f32)`
definitions that mutate a scoped `egui::Ui` with the `f32` normalized progression.

## Example

The following demonstrates a simple fade transition between "`Page`" `enum` variants.
[`animate`](crate::anim_ops::animate) persists the prior `state` variable for the duration of
the *out* animation segment then passes the next variant for the *in* segment. See the
documention of [`Animation`](crate::anim::Animation) for more.

```rust
// A `0.5` second fade out/in animation.
const FADE_ANIM: Animation<Sequence> = Animation::new(0.5, fade, fade);

#[derive(Default, Clone, Copy, PartialEq)]
enum Page {
    #[default]
    Title,
    Page0,
}
// The mutable state variable.
let mut state = Page::Title;

// Animate on change of the `state` variable.
animate(ui, "page_anim", state, FADE_ANIM,
    |ui, page| match page {
        // Display the `Title` interface.
        Page::Title => {
            if ui.button("Next").clicked() {
                state = Page::Page0;
            }
        }
        // Display the `Page0` interface.
        Page::Page0 => {
            if ui.button("Back").clicked() {
                state = Page::Title;
            }
        }
    },
);
```

## Operations

The primary operations offered by this crate are [`animate`](anim_ops::animate) and [`run_state`](anim_ops::run_state).
As demonstrated in the earlier example, [`animate`](anim_ops::animate) is used to pass an
interface to a child scope and animate the inner ui elements on change. [`run_state`](anim_ops::run_state)
can be used to inspect the animation state, usually outside of the [`animate`](anim_ops::animate)
function. This can be useful for disabling elements that may affect the state for the duration
of an animation.

The *normal* operations, [`fade`](norm_ops::fade), [`translate`](norm_ops::translate) and [`scale`](norm_ops::scale)
are included to assist in defining an animation.

## Compatibility

egui | egui_animate
---|---
0.34 | 0.4
0.33 | 0.3
0.32 | 0.2

License: MIT OR Apache-2.0
