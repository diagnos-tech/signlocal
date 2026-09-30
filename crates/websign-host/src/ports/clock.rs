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

/// Whole seconds since the Unix epoch, for records and validity checks;
/// instants before 1970 or beyond `i64` clamp to 0.
pub fn unix_seconds(time: SystemTime) -> i64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
        .unwrap_or(0)
}
