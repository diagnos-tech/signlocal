//! The confirmation window's state machine (`docs/ux.md` §4.8).
//!
//! Inputs: host commands ([`ConfirmModel::apply`]) and person input
//! ([`ConfirmModel::input`]). Outputs: what to draw ([`ConfirmModel::view`])
//! and what the person decided ([`Intent`]). Time is always passed in, so
//! every rule (arming, the 900 ms success hold, the countdown) is testable
//! without sleeping.

use std::time::Instant;

use websign_core::Fingerprint;
use websign_protocol::ErrorCode;
use websign_protocol::messages::DiagnosticsTab;

use super::arming::Arming;
use super::port::UiCommand;
use super::view::ConfirmView;

/// The states of `docs/ux.md` §4.8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmState {
    /// No request on screen.
    Idle,
    LoadingCerts,
    /// No usable certificate; possible certificates shown.
    Empty,
    /// A certificate is selected; waiting for Continue (D11) or the digest.
    Choosing,
    /// The code is visible; the primary button arms.
    Ready,
    PinError,
    PinLocked,
    Signing,
    /// Success hold (900 ms), then the next request or hide.
    Success,
    /// A recoverable or final error banner.
    Error {
        code: ErrorCode,
    },
    /// "{site} cancelled the request." for 1.5 s.
    SiteCancelled,
    Timeout,
}

/// Something the person did, already translated from egui input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserInput {
    /// The window became visible and focused, or lost focus.
    Focus(bool),
    Select(Fingerprint),
    /// Pointer down / up on the primary button.
    PrimaryPress,
    PrimaryRelease,
    /// Enter in the PIN field or on the focused primary button.
    Enter,
    Escape,
    CancelButton,
    CloseButton,
    /// The PIN field's length changed (the value never reaches the model).
    PinLength(usize),
    Remember(bool),
    Rescan,
    OpenDiagnostics,
    /// "Try through the token driver" (`docs/ux.md` §5.11).
    UseAlternatePath,
}

/// A decision the app turns into a [`super::port::UiEvent`] (adding the PIN
/// from its field when the intent is `Sign`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Selected(Fingerprint),
    Continue(Fingerprint),
    Sign {
        fingerprint: Fingerprint,
        via: usize,
        remember: bool,
    },
    Choose {
        fingerprint: Fingerprint,
        remember: bool,
    },
    Cancel(ErrorCode),
    Rescan,
    OpenDiagnostics(Option<DiagnosticsTab>),
}

/// The window's whole state.
#[derive(Debug, Clone)]
pub struct ConfirmModel {
    state: ConfirmState,
    arming: Arming,
}

impl Default for ConfirmModel {
    fn default() -> Self {
        ConfirmModel::new()
    }
}

impl ConfirmModel {
    /// An idle window.
    pub fn new() -> ConfirmModel {
        ConfirmModel {
            state: ConfirmState::Idle,
            arming: Arming::default(),
        }
    }

    /// The current state.
    pub fn state(&self) -> &ConfirmState {
        &self.state
    }

    /// Applies a host command.
    pub fn apply(&mut self, command: UiCommand, now: Instant) {
        let _ = (command, now);
        todo!("SPEC.md §2.2")
    }

    /// Applies person input; returns the decisions it produced (usually none
    /// or one).
    pub fn input(&mut self, input: UserInput, now: Instant) -> Vec<Intent> {
        let _ = (input, now);
        todo!("SPEC.md §2.2")
    }

    /// Advances timers (success hold, site-cancelled hold, countdown) and
    /// returns decisions they produced.
    pub fn tick(&mut self, now: Instant) -> Vec<Intent> {
        let _ = now;
        todo!("SPEC.md §2.2")
    }

    /// What to draw at `now`.
    pub fn view(&self, now: Instant) -> ConfirmView {
        let _ = (now, &self.arming);
        todo!("SPEC.md §2.4")
    }

    /// The next instant something changes without input (arming completes,
    /// a hold ends, the countdown ticks), for `request_repaint_after`.
    pub fn next_deadline(&self, now: Instant) -> Option<Instant> {
        let _ = now;
        todo!("SPEC.md §2.2")
    }
}
