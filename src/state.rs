/// Animation state progress.
pub struct AnimProgress {
    start_time: f64,
    current_time: f64,
}

impl AnimProgress {
    /// Create a new `AnimProgress` from a starting and current time.
    pub(super) const fn new(start_time: f64, current_time: f64) -> Self {
        Self {
            start_time,
            current_time,
        }
    }

    /// Get animation start time.
    #[inline]
    pub(super) const fn start(&self) -> f64 {
        self.start_time
    }

    /// Get the elapsed time. Returns `Some(0.0)` if the animation has yet to begin, and `None` if
    /// the animation has finished.
    pub(super) fn elapsed(&self, duration: f64) -> Option<f32> {
        let elapsed = (self.current_time - self.start()).max(0.0);
        (elapsed < duration).then_some(elapsed as f32)
    }

    /// Get the elapsed normal. Returns `Some(0.0)` if the animation has yet to begin, and `None` if
    /// the animation has finished.
    pub(super) fn elapsed_normal(&self, duration: f64) -> Option<f32> {
        self.elapsed(duration)
            .map(|elapsed| elapsed / duration as f32)
    }

    /// Offset the animation start time by the given amount.
    pub(super) const fn offset(&self, offset: f64) -> Self {
        Self {
            start_time: self.start_time + offset,
            current_time: self.current_time,
        }
    }
}

/// Animation state values.
pub struct AnimValues<T> {
    start_val: T,
    current_val: T,
}

impl<T> AnimValues<T> {
    /// Create a new `AnimVars` from the a starting and current value.
    pub(super) const fn new(start_val: T, current_val: T) -> Self {
        Self {
            start_val,
            current_val,
        }
    }

    /// Consume `self`, returning the starting value.
    pub(super) fn start_value(self) -> T {
        self.start_val
    }

    /// Consume `self`, returning the current value.
    pub(super) fn current_value(self) -> T {
        self.current_val
    }

    /// Consume `self`, returning a tuple of the respective start and current value.
    pub(super) fn split(self) -> (T, T) {
        (self.start_val, self.current_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod animation_state {
        use super::*;

        const TEST_ANIM_STATE: AnimProgress = AnimProgress::new(1.0, 1.0);

        #[test]
        fn test_elapsed() {
            let mut state = TEST_ANIM_STATE;

            assert_eq!(state.elapsed(1.0), Some(0.0));
            state.current_time = 1.5;
            assert_eq!(state.elapsed(1.0), Some(0.5));
            state.current_time = 2.0;
            assert_eq!(state.elapsed(1.0), None);
            state.current_time = 3.0;
            assert_eq!(state.elapsed(1.0), None);
        }

        #[test]
        fn test_elapsed_normal() {
            let mut state = TEST_ANIM_STATE;

            assert_eq!(state.elapsed_normal(1.0), Some(0.0));
            state.current_time = 1.75;
            assert_eq!(state.elapsed_normal(1.0), Some(0.75));
            state.current_time = 3.0;
            assert_eq!(state.elapsed_normal(1.0), None);
            state.current_time = 4.0;
            assert_eq!(state.elapsed_normal(1.0), None);
        }
    }
}
