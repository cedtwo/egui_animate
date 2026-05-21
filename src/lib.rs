#![doc = include_str!("../README.md")]
mod mem;
mod ty;

mod anim;
mod state;

pub use anim::{Animation, AnimationSegment};
pub use state::{RunState, animate, run_state};
