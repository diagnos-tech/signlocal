//! How a request ends: failures the person can act on, and results.

use std::time::{Duration, Instant};

use super::machine::{ConfirmModel, ConfirmState};
use super::pin::pin_error_of;
use super::port::{Failure, Finish};
use super::view::banner_code;
use crate::certs::{CertRow, DisabledReason, RowStatus};

/// The success screen stays this long; the result is already sent.
pub(super) const SUCCESS_HOLD: Duration = Duration::from_millis(900);
/// "{site} cancelled the request." and "The request expired" stay this long
/// (`docs/ux.md` §4.8), long enough to read why the window is closing.
pub(super) const NOTICE_HOLD: Duration = Duration::from_millis(1500);

/// Failures whose banner offers "Try again" with the same certificate
/// (`docs/ux.md` §15): a token plugged back in or a driver hiccup can
/// succeed on a second attempt. `Internal` offers "Copy details" and
/// "Open diagnostics" instead: repeating an unexpected failure only repeats
/// it.
pub(super) fn is_retryable(failure: &Failure) -> bool {
    matches!(
        failure,
        Failure::TokenRemoved | Failure::DriverFailure { .. }
    )
}

impl ConfirmModel {
    pub(super) fn failed(&mut self, failure: Failure) {
        if !self.on_screen() {
            return;
        }
        match &failure {
            Failure::PinIncorrect {
                count_low,
                final_try,
            } => {
                self.pin_error = Some(pin_error_of(*count_low, *final_try));
                self.pin_len = 0;
                self.banner = None;
                self.state = ConfirmState::PinError;
            }
            Failure::PinLocked { .. } => {
                self.disable_selected(DisabledReason::PinLocked);
                self.pin_error = None;
                self.banner = Some(failure);
                self.state = ConfirmState::PinLocked;
            }
            _ => {
                if matches!(failure, Failure::UnsupportedAlgorithm { .. }) {
                    self.disable_selected(DisabledReason::Incompatible);
                }
                self.pin_error = None;
                self.state = ConfirmState::Error {
                    code: banner_code(&failure),
                };
                self.banner = Some(failure);
            }
        }
    }

    pub(super) fn finished(&mut self, finish: Finish, now: Instant) {
        if matches!(self.state, ConfirmState::Idle | ConfirmState::Success) {
            return;
        }
        match finish {
            Finish::Signed | Finish::Chosen => {
                self.state = ConfirmState::Success;
                self.hold_until = Some(now + SUCCESS_HOLD);
            }
            Finish::SiteCancelled => {
                self.state = ConfirmState::SiteCancelled;
                self.hold_until = Some(now + NOTICE_HOLD);
            }
            Finish::Timeout => {
                self.state = ConfirmState::Timeout;
                self.hold_until = Some(now + NOTICE_HOLD);
            }
            Finish::Aborted => self.reset(),
        }
    }

    /// Marks the selected row as unable to sign, in place, so the selection
    /// does not jump under the pointer.
    fn disable_selected(&mut self, reason: DisabledReason) {
        let Some(list) = &mut self.list else { return };
        let Some(selected) = list.selected else {
            return;
        };
        let mark = |row: &mut CertRow| {
            if row.candidate.fingerprint == selected {
                row.status = RowStatus::Disabled(reason);
            }
        };
        list.usable.iter_mut().for_each(mark);
        list.disabled.iter_mut().for_each(mark);
    }
}
