//! Small questions the transitions ask of the model.

use std::time::Instant;

use websign_core::Fingerprint;

use super::machine::{ConfirmModel, ConfirmState};
use super::port::{Mode, RequestKey};
use super::slot::CodeSlot;
use crate::certs::{CertRow, RowStatus};

impl ConfirmModel {
    /// Forgets the request and returns to `Idle`. Focus and arming survive:
    /// they belong to the window, not to the request.
    pub(super) fn reset(&mut self) {
        let (arming, focused) = (self.arming, self.focused);
        *self = ConfirmModel::new();
        self.arming = arming;
        self.focused = focused;
    }

    /// Restarts the arming delay because new content sits under the pointer.
    /// The 600 ms count from the window being visible *and* focused
    /// (`docs/ux.md` §4.7), so an unfocused window stays unarmed until
    /// `Focus(true)`: a click that only brings it forward never approves.
    pub(super) fn rearm(&mut self, now: Instant) {
        if self.arming.is_armed(now) {
            self.settled = true;
        }
        if self.focused {
            self.arming.rearm(now);
        } else {
            self.arming.disarm();
        }
    }

    /// Whether `key` is the request on screen.
    pub(super) fn is_current(&self, key: RequestKey) -> bool {
        self.request
            .as_ref()
            .is_some_and(|request| request.key == key)
    }

    /// A request is waiting for the person (no result has been shown).
    pub(super) fn on_screen(&self) -> bool {
        !matches!(
            self.state,
            ConfirmState::Idle
                | ConfirmState::Success
                | ConfirmState::SiteCancelled
                | ConfirmState::Timeout
        )
    }

    pub(super) fn mode(&self) -> Option<Mode> {
        self.request.as_ref().map(|request| request.mode)
    }

    /// Whether the caller's consent covers `fingerprint`: the host
    /// releases it without Continue.
    pub(super) fn is_consented(&self, fingerprint: Option<Fingerprint>) -> bool {
        let (Some(request), Some(fingerprint)) = (&self.request, fingerprint) else {
            return false;
        };
        request.consented.contains(&fingerprint)
    }

    /// "Remember" can still add something: the caller may be remembered
    /// and the selected certificate is not covered yet.
    pub(super) fn may_remember(&self) -> bool {
        let selected = self.list.as_ref().and_then(|list| list.selected);
        self.request
            .as_ref()
            .is_some_and(|request| request.can_remember)
            && !self.is_consented(selected)
    }

    /// The selected row, wherever it sits.
    pub(super) fn selected_row(&self) -> Option<&CertRow> {
        let list = self.list.as_ref()?;
        let selected = list.selected?;
        list.usable
            .iter()
            .chain(&list.disabled)
            .find(|row| row.candidate.fingerprint == selected)
    }

    /// The selection, when its row can sign.
    pub(super) fn selected_usable(&self) -> Option<Fingerprint> {
        self.selected_row()
            .filter(|row| row.status == RowStatus::Usable)
            .map(|row| row.candidate.fingerprint)
    }

    /// Enters `Choosing` with the code card in its starting state: choose
    /// mode has no code, the digest of a consented certificate is already on
    /// its way, any other certificate needs Continue first.
    pub(super) fn enter_choosing(&mut self, now: Instant) {
        self.state = ConfirmState::Choosing;
        self.chosen = false;
        let selected = self.list.as_ref().and_then(|list| list.selected);
        self.code = match self.mode() {
            Some(Mode::Sign { .. }) if self.is_consented(selected) => CodeSlot::Preparing(now),
            Some(Mode::Sign { .. }) => CodeSlot::Hint,
            _ => CodeSlot::None,
        };
    }

    /// Signing may start: a usable selection, its code on screen and a PIN
    /// of acceptable length when our field is shown.
    pub(super) fn can_sign(&self) -> bool {
        self.selected_usable().is_some()
            && matches!(self.code, CodeSlot::Ready(_))
            && self.pin_acceptable()
    }
}
