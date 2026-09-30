//! The person acts in the window.

use websign_core::Fingerprint;
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::UiEvent;

use super::{SignFlow, SignState};
use crate::flow::Effect;
use crate::flow::errors::{error_reply, finished};

impl SignFlow {
    /// The person acted in the window.
    pub fn on_ui(&mut self, event: UiEvent) -> Vec<Effect> {
        match event {
            UiEvent::Selected { key, fingerprint } if key == self.key => self.selected(fingerprint),
            UiEvent::Continue { key, fingerprint } if key == self.key => {
                self.continued(fingerprint)
            }
            UiEvent::Sign {
                key,
                fingerprint,
                via,
                pin,
                remember,
            } if key == self.key => self.sign(fingerprint, via, pin, remember),
            UiEvent::Cancel { key, code } if key == self.key => self.cancelled(code),
            _ => Vec::new(),
        }
    }

    /// The selection moved. A remembered caller gets the new certificate's
    /// digest request at once; anyone else must press "Continue" again,
    /// because each certificate is its own disclosure (D11).
    fn selected(&mut self, fingerprint: Fingerprint) -> Vec<Effect> {
        let holds_it = match &self.state {
            SignState::AwaitingDigest {
                fingerprint: held, ..
            }
            | SignState::Ready {
                fingerprint: held, ..
            } => *held == fingerprint,
            SignState::Selecting { .. } => false,
            _ => return Vec::new(),
        };
        if holds_it || !self.is_usable(&fingerprint) {
            return Vec::new();
        }
        if self.remembered {
            self.release(fingerprint)
        } else {
            self.state = SignState::Selecting {
                selected: Some(fingerprint),
            };
            Vec::new()
        }
    }

    /// "Continue": the person agrees to release this certificate.
    fn continued(&mut self, fingerprint: Fingerprint) -> Vec<Effect> {
        if !matches!(self.state, SignState::Selecting { .. }) || !self.is_usable(&fingerprint) {
            return Vec::new();
        }
        self.release(fingerprint)
    }

    /// Cancel, Esc or close: the window chose the code (`docs/ux.md` §15).
    fn cancelled(&mut self, code: ErrorCode) -> Vec<Effect> {
        if !self.finish_now() {
            return Vec::new();
        }
        vec![error_reply(code), finished(self.key, code)]
    }

    fn is_usable(&self, fingerprint: &Fingerprint) -> bool {
        self.listing
            .as_ref()
            .is_some_and(|listing| listing.usable(fingerprint).is_some())
    }
}
