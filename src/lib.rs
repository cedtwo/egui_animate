#![doc = include_str!("../README.md")]
mod mem;
mod ty;

mod anim;
mod state;

pub use anim::{Animation, AnimationSegment, RunState};
pub use state::{animate, run_state};
