//! The decisions only the primary button and Enter can produce: Continue
//! (D11), Sign, Try again, the alternate path and Choose (`docs/ux.md` §4.7,
//! §4.9, §5.11). Callers check arming first; these only check that the
//! window is in a state where the decision makes sense.

use std::time::Instant;

use websign_core::Fingerprint;

use super::machine::{ConfirmModel, ConfirmState, Intent};
use super::outcome::is_retryable;
use super::port::{Failure, Mode};
use super::slot::CodeSlot;
use super::view::PinBlock;

impl ConfirmModel {
    /// Enter in the PIN field or on the focused Sign button. Enter only ever
    /// signs: it never releases a certificate (Continue) nor chooses one,
    /// because before the code is on screen focus sits on the list
    /// (`docs/ux.md` §4.9: never on the primary button) and an Enter there
    /// only moves focus. Releasing takes a click, or Space on the focused
    /// button, which the app reports as a press and a release.
    pub(super) fn enter(&mut self) -> Vec<Intent> {
        if matches!(self.mode(), Some(Mode::Sign { .. })) {
            self.activate_sign()
        } else {
            Vec::new()
        }
    }

    /// A valid click on the primary button.
    pub(super) fn activate(&mut self, now: Instant) -> Vec<Intent> {
        let (Some(mode), Some(fingerprint)) = (self.mode(), self.selected_usable()) else {
            return Vec::new();
        };
        match (mode, &self.state) {
            (Mode::Choose, ConfirmState::Choosing) if !self.chosen => {
                self.chosen = true;
                vec![Intent::Choose {
                    fingerprint,
                    remember: self.remember_wanted(),
                }]
            }
            (Mode::Sign { .. }, ConfirmState::Choosing) if self.code == CodeSlot::Hint => {
                self.code = CodeSlot::Preparing(now);
                vec![Intent::Continue(fingerprint)]
            }
            (Mode::Sign { .. }, _) => self.activate_sign(),
            _ => Vec::new(),
        }
    }

    /// Sign from `Ready` or after a wrong PIN, or "Try again" after a
    /// recoverable error.
    fn activate_sign(&mut self) -> Vec<Intent> {
        let Some(fingerprint) = self.selected_usable().filter(|_| self.can_sign()) else {
            return Vec::new();
        };
        let allowed = match self.state {
            ConfirmState::Ready | ConfirmState::PinError => true,
            ConfirmState::Error { .. } => self.banner.as_ref().is_some_and(is_retryable),
            _ => false,
        };
        if allowed {
            self.begin_signing(fingerprint)
        } else {
            Vec::new()
        }
    }

    fn begin_signing(&mut self, fingerprint: Fingerprint) -> Vec<Intent> {
        self.state = ConfirmState::Signing;
        self.pin_error = None;
        self.banner = None;
        vec![Intent::Sign {
            fingerprint,
            via: self.via,
            remember: self.remember_wanted(),
        }]
    }

    /// "Remember this site" only counts where it can still add the chosen
    /// certificate to the caller's consent.
    fn remember_wanted(&self) -> bool {
        self.remember && self.may_remember()
    }

    /// "Try through the token driver": sign again over the first alternate
    /// path. A driver that needs our PIN field (the failed OS store asked
    /// through its own dialog, so none was typed) first shows the field:
    /// the window returns to `Ready` on that path, re-armed, and Sign sends
    /// the PIN. Otherwise (PIN pad, unlocked token) it signs at once.
    pub(super) fn use_alternate_path(&mut self, now: Instant) -> Vec<Intent> {
        let offered = matches!(
            self.banner,
            Some(Failure::DriverFailure {
                alternate: true,
                ..
            })
        );
        let Some(fingerprint) = self
            .selected_usable()
            .filter(|_| offered && matches!(self.code, CodeSlot::Ready(_)))
        else {
            return Vec::new();
        };
        self.via = 1;
        if matches!(self.pin_block(), PinBlock::Field { .. }) {
            self.state = ConfirmState::Ready;
            self.banner = None;
            self.pin_error = None;
            self.pin_len = 0;
            self.rearm(now);
            return Vec::new();
        }
        if !self.can_sign() {
            return Vec::new();
        }
        self.begin_signing(fingerprint)
    }
}
