#![doc = include_str!("../README.md")]
mod mem;
mod ty;

mod segment;

mod anim;
mod state;

pub use anim::{Animation, RunState};
pub use segment::AnimationSegment;
pub use state::{animate, run_state};
