//! Time-driven changes: the result holds, the arming delay, the countdown
//! and the skeleton delay.

use std::time::{Duration, Instant};

use super::machine::{ConfirmModel, ConfirmState, Intent};
use super::slot::{CodeSlot, SKELETON_DELAY};

/// The footer counts down during the last stretch of the request.
const COUNTDOWN: Duration = Duration::from_secs(30);

/// Closes the result screens once their hold is over.
pub(super) fn tick(model: &mut ConfirmModel, now: Instant) -> Vec<Intent> {
    let holding = matches!(
        model.state,
        ConfirmState::Success | ConfirmState::SiteCancelled | ConfirmState::Timeout
    );
    if holding && model.hold_until.is_some_and(|end| now >= end) {
        model.reset();
    }
    Vec::new()
}

/// The earliest instant after `now` at which [`ConfirmModel::view`] changes
/// without input.
pub(super) fn next_deadline(model: &ConfirmModel, now: Instant) -> Option<Instant> {
    let mut candidates = Vec::new();
    if model.state != ConfirmState::Idle {
        candidates.extend(model.arming.armed_at());
    }
    if !model.on_screen() {
        candidates.extend(model.hold_until);
    }
    if model.on_screen() {
        candidates.extend(countdown_change(model, now));
        if let CodeSlot::Preparing(since) = model.code {
            candidates.push(since + SKELETON_DELAY);
        }
    }
    candidates.into_iter().filter(|at| *at > now).min()
}

/// Whole seconds shown by the footer countdown, once inside the last 30 s.
pub(super) fn seconds_left(model: &ConfirmModel, now: Instant) -> Option<u32> {
    let left = model.deadline?.saturating_duration_since(now);
    (left <= COUNTDOWN).then(|| {
        let whole = left.as_secs() + u64::from(left.subsec_nanos() > 0);
        u32::try_from(whole).unwrap_or(u32::MAX)
    })
}

/// When the countdown starts, or when its displayed second next changes.
fn countdown_change(model: &ConfirmModel, now: Instant) -> Option<Instant> {
    let deadline = model.deadline?;
    let left = deadline.saturating_duration_since(now);
    if left > COUNTDOWN {
        return deadline.checked_sub(COUNTDOWN);
    }
    let shown = u64::from(seconds_left(model, now)?);
    // The number drops when the time left falls below `shown - 1` seconds,
    // or below `shown` when it is an exact number of seconds.
    let boundary = if left.subsec_nanos() == 0 {
        shown.checked_sub(1)?
    } else {
        shown - 1
    };
    deadline.checked_sub(Duration::from_secs(boundary))
}
