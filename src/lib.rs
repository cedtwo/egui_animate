#![doc = include_str!("../README.md")]
mod mem;
mod ty;

mod anim;
mod animate;
mod segment;
mod state;

mod ops;

pub use anim::{Animation, RunState};
pub use ops::{animate, run_state};
pub use segment::AnimationSegment;
