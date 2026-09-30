//! The anti-accident arming of the primary button (`docs/ux.md` §4.7).
//!
//! The button accepts a click only when the press **and** the release happen
//! after [`ARMING_DELAY`] of the window being visible and focused, measured
//! from the last event that re-arms (focus regained, certificate changed,
//! digest changed, next request in the queue). Keys other than Esc are
//! ignored while unarmed.

use std::time::{Duration, Instant};

/// Longer than the default double-click interval (500 ms on Windows and macOS).
pub const ARMING_DELAY: Duration = Duration::from_millis(600);

/// Arming state of one window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Arming {
    since: Option<Instant>,
    pressed_armed: bool,
}

impl Arming {
    /// Starts (or restarts) the delay at `now`. Call when the window becomes
    /// visible and focused, and on every re-arm event.
    pub fn rearm(&mut self, now: Instant) {
        let _ = now;
        todo!("SPEC.md §2.1")
    }

    /// Stops counting (focus lost, window hidden).
    pub fn disarm(&mut self) {
        todo!("SPEC.md §2.1")
    }

    /// Whether input is accepted at `now`.
    pub fn is_armed(&self, now: Instant) -> bool {
        let _ = now;
        todo!("SPEC.md §2.1")
    }

    /// The press half of a click; remembers whether it was armed.
    pub fn press(&mut self, now: Instant) {
        let _ = now;
        todo!("SPEC.md §2.1")
    }

    /// The release half; `true` = the click counts.
    pub fn release(&mut self, now: Instant) -> bool {
        let _ = now;
        todo!("SPEC.md §2.1")
    }

    /// When the button becomes armed, for scheduling a repaint.
    pub fn armed_at(&self) -> Option<Instant> {
        todo!("SPEC.md §2.1")
    }
}
