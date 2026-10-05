/// Schedules [`Animation`](crate::Animation) segments to run sequentially. This will run the
/// respective *out* and *in* segment sequentially for a duration of the sum of both segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sequence {}

/// Schedules [`Animation`](crate::Animation) segments to run simultaneously. This will run the
/// respective *out* and *in* segments at the same time (and in the same region) for their
/// respective duration. The *out* and *in* segments of a `Layer` animation start begin and end at
/// their respective `1.0` value with the total duration being the greater of each individual
/// duration. Therefore an *in* segment of a duraction shorter than an *out* segment will begin
/// later than the *out* segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {}
