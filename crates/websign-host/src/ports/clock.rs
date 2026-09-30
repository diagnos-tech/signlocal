//! Time, injectable.

use std::time::{Instant, SystemTime};

/// Monotonic time for deadlines, wall time for records and validity.
pub trait Clock {
    fn now(&self) -> Instant;
    fn wall(&self) -> SystemTime;
}

/// The real clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn wall(&self) -> SystemTime {
        SystemTime::now()
    }
}
