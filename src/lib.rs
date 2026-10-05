#![doc = include_str!("../README.md")]
mod mem;

mod animate;
mod schedule;
mod segment;
mod state;

mod anim;

mod anim_ops;
mod norm_ops;

/// Core types and operations.
pub mod prelude {
    pub use crate::anim::Animation;
    pub use crate::anim::LayerRunState;
    pub use crate::anim::SequenceRunState;

    pub use crate::schedule::{Layer, Sequence};
    pub use crate::segment::AnimationSegment;

    pub use crate::anim_ops::{animate, run_state};
    pub use crate::norm_ops::{fade, scale, translate};
}
