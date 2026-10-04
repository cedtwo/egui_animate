/// Schedules [`Animation`](crate::Animation) segments to run sequentially. This will run the
/// respective *out* and *in* segment sequentially for a duration of the sum of both segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sequence {}

/// Schedules [`Animation`](crate::Animation) segments to run simultaneously. This will run the
/// respective *out* and *in* segments at the same time (and in the same region) for their
/// respective duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {}
