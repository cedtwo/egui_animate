#![doc = include_str!("../README.md")]
mod mem;

mod animate;
mod schedule;
mod segment;
mod state;

mod anim;
mod ops;

pub use anim::Animation;
pub use anim::LayerRunState;
pub use anim::SequenceRunState;

pub use schedule::{Layer, Sequence};
pub use segment::AnimationSegment;

pub use ops::{animate, run_state};
