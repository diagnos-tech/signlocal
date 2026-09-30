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
        let arming = self.arming;
        *self = ConfirmModel::new();
        self.arming = arming;
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

    pub(super) fn is_remembered(&self) -> bool {
        self.request
            .as_ref()
            .is_some_and(|request| request.remembered)
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
    /// mode has no code, a remembered caller's digest is already on its way,
    /// anyone else must press Continue first.
    pub(super) fn enter_choosing(&mut self, now: Instant) {
        self.state = ConfirmState::Choosing;
        self.chosen = false;
        self.code = match self.mode() {
            Some(Mode::Sign { .. }) if self.is_remembered() => CodeSlot::Preparing(now),
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
