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
        self.since = Some(now);
        self.pressed_armed = false;
    }

    /// Stops counting (focus lost, window hidden).
    pub fn disarm(&mut self) {
        self.since = None;
        self.pressed_armed = false;
    }

    /// Whether input is accepted at `now`.
    pub fn is_armed(&self, now: Instant) -> bool {
        self.since
            .is_some_and(|since| now.saturating_duration_since(since) >= ARMING_DELAY)
    }

    /// The press half of a click; remembers whether it was armed.
    pub fn press(&mut self, now: Instant) {
        self.pressed_armed = self.is_armed(now);
    }

    /// The release half; `true` = the click counts.
    pub fn release(&mut self, now: Instant) -> bool {
        let counts = self.pressed_armed && self.is_armed(now);
        self.pressed_armed = false;
        counts
    }

    /// When the button becomes armed, for scheduling a repaint.
    pub fn armed_at(&self) -> Option<Instant> {
        self.since.map(|since| since + ARMING_DELAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn arms_after_the_delay() {
        let t0 = Instant::now();
        let mut arming = Arming::default();
        assert!(!arming.is_armed(t0));
        assert_eq!(arming.armed_at(), None);
        arming.rearm(t0);
        assert!(!arming.is_armed(t0 + MS(599)));
        assert!(arming.is_armed(t0 + MS(600)));
        assert_eq!(arming.armed_at(), Some(t0 + MS(600)));
    }

    #[test]
    fn press_before_arming_never_counts() {
        let t0 = Instant::now();
        let mut arming = Arming::default();
        arming.rearm(t0);
        arming.press(t0 + MS(100));
        assert!(!arming.release(t0 + MS(700)));
    }

    #[test]
    fn armed_click_counts_once() {
        let t0 = Instant::now();
        let mut arming = Arming::default();
        arming.rearm(t0);
        arming.press(t0 + MS(650));
        assert!(arming.release(t0 + MS(700)));
        assert!(!arming.release(t0 + MS(710)));
    }

    #[test]
    fn rearm_or_disarm_between_press_and_release_cancels_the_click() {
        let t0 = Instant::now();
        let mut arming = Arming::default();
        arming.rearm(t0);
        arming.press(t0 + MS(650));
        arming.rearm(t0 + MS(660));
        assert!(!arming.release(t0 + MS(1400)));
        arming.press(t0 + MS(1400));
        arming.disarm();
        assert!(!arming.release(t0 + MS(2000)));
        assert!(!arming.is_armed(t0 + MS(2000)));
    }
}
