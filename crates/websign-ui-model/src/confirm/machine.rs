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
use super::port::{Failure, OpenRequest, UiCommand};
use super::slot::CodeSlot;
use super::view::{ConfirmView, PinError};
use super::{build, commands, inputs, timers};
use crate::certs::CertList;
use crate::possible::PossibleCard;

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
    /// "The request expired" for 1.5 s.
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
    /// Enter in the PIN field or on the focused primary button. The app never
    /// sends it for Enter on a list row, which only moves focus (`docs/ux.md`
    /// §4.9); the model accepts it only as "Sign".
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
///
/// Fields are visible to the sibling modules that implement its transitions
/// ([`commands`], [`inputs`], [`timers`], [`build`]); nothing outside
/// `confirm` can touch them.
#[derive(Debug, Clone)]
pub struct ConfirmModel {
    pub(super) state: ConfirmState,
    pub(super) arming: Arming,
    pub(super) request: Option<OpenRequest>,
    pub(super) list: Option<CertList>,
    pub(super) possible: Vec<PossibleCard>,
    pub(super) code: CodeSlot,
    /// Characters typed in the PIN field (the value never reaches the model).
    pub(super) pin_len: usize,
    pub(super) pin_error: Option<PinError>,
    pub(super) remember: bool,
    pub(super) banner: Option<Failure>,
    /// Signing path: 0 = primary, 1 = first alternate.
    pub(super) via: usize,
    /// When the request times out.
    pub(super) deadline: Option<Instant>,
    /// When the success, site-cancelled or timeout notice closes.
    pub(super) hold_until: Option<Instant>,
    /// The window has keyboard focus. Arming only counts while it does
    /// (`docs/ux.md` §4.7), so a request that opens behind another window
    /// stays unarmed until the person brings it forward.
    pub(super) focused: bool,
    /// Choose mode: the decision was sent and the answer is pending.
    pub(super) chosen: bool,
    /// The window has been armed since it last gained focus or took a
    /// request: the person has had it in front of them for 600 ms. From then
    /// on moving the selection is not gated (a selection approves nothing and
    /// re-arms the button), so ↑/↓ are not limited to one step per 600 ms,
    /// while keys and clicks that land on a window just shown still are.
    pub(super) settled: bool,
    /// When the current listing started (`LoadingCerts`).
    pub(super) loading_since: Option<Instant>,
    /// The device the host says is slow to list ("Still reading {device}").
    pub(super) slow_device: Option<String>,
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
            request: None,
            list: None,
            possible: Vec::new(),
            code: CodeSlot::None,
            pin_len: 0,
            pin_error: None,
            remember: false,
            banner: None,
            via: 0,
            deadline: None,
            hold_until: None,
            focused: false,
            chosen: false,
            settled: false,
            loading_since: None,
            slow_device: None,
        }
    }

    /// The current state.
    pub fn state(&self) -> &ConfirmState {
        &self.state
    }

    /// Applies a host command.
    pub fn apply(&mut self, command: UiCommand, now: Instant) {
        commands::apply(self, command, now);
    }

    /// Applies person input; returns the decisions it produced (usually none
    /// or one).
    pub fn input(&mut self, input: UserInput, now: Instant) -> Vec<Intent> {
        inputs::handle(self, input, now)
    }

    /// Closes a result notice whose hold is over. Returns the decisions the
    /// timers produced; none today, since every result is sent before its
    /// notice shows, but the renderer's loop stays the same if one appears.
    pub fn tick(&mut self, now: Instant) -> Vec<Intent> {
        timers::tick(self, now)
    }

    /// Whether the window takes decisions at `now` (`docs/ux.md` §4.7):
    /// focused, and 600 ms past the last re-arm. The model gates its own
    /// inputs with it; the renderer asks too, to drop keystrokes before any
    /// widget sees them, so typing meant for the site never fills the PIN
    /// field.
    pub fn is_armed(&self, now: Instant) -> bool {
        self.arming.is_armed(now)
    }

    /// Whether [`UserInput::Select`] is taken at `now`: once armed, or
    /// after the window has been armed since it last gained focus or took a
    /// request. The renderer lets the list's navigation keys through its
    /// keystroke guard while this holds.
    pub fn accepts_selection(&self, now: Instant) -> bool {
        self.settled || self.is_armed(now)
    }

    /// What to draw at `now`.
    pub fn view(&self, now: Instant) -> ConfirmView {
        build::view(self, now)
    }

    /// The next instant something changes without input (arming completes,
    /// a hold ends, the countdown ticks), for `request_repaint_after`.
    pub fn next_deadline(&self, now: Instant) -> Option<Instant> {
        timers::next_deadline(self, now)
    }
}

#[cfg(test)]
mod edge_tests;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod pin_and_list_tests;
#[cfg(test)]
mod rig;
#[cfg(test)]
mod selection_tests;
#[cfg(test)]
mod tests;
