//! The waits a sign request can run out of, besides the person's decision.

use std::time::Instant;

use websign_protocol::{AppMessage, ErrorCode, WireError};
use websign_ui_model::confirm::port::{Finish, UiCommand};

use super::{SignFlow, SignState};
use crate::flow::Effect;

impl SignFlow {
    /// The decision deadline that applies now. None while the key store
    /// signs: the person has decided, and an OS PIN dialog or a slow token
    /// must not be cut off (the caller can still `cancel`).
    pub fn deadline_now(&self) -> Option<Instant> {
        match self.state {
            SignState::Signing { .. } => None,
            _ => self.deadline,
        }
    }

    /// [`super::CHAIN_WAIT`] passed in `Releasing`: `sign.need_digest` goes
    /// without the chain (the `sign.result` still carries it when the
    /// lookup finishes before the signature).
    pub fn chain_wait_over(&mut self) -> Vec<Effect> {
        match self.state {
            SignState::Releasing { fingerprint, .. } => self.need_digest(fingerprint),
            _ => Vec::new(),
        }
    }

    /// The caller did not answer `sign.need_digest` within
    /// `DIGEST_TIMEOUT`: the request ends with `Timeout`, and the window
    /// says the site did not prepare the document, not that nobody decided.
    pub fn digest_timed_out(&mut self) -> Vec<Effect> {
        let on_screen = self.on_screen();
        if !self.finish_now() {
            return Vec::new();
        }
        let mut effects = vec![Effect::Send(AppMessage::Error(WireError {
            code: ErrorCode::Timeout,
            message: "the caller did not send the digest in time".to_owned(),
            details: None,
        }))];
        if on_screen {
            effects.push(Effect::Ui(UiCommand::Finished {
                key: self.key,
                finish: Finish::DigestTimeout,
            }));
        }
        effects
    }
}
