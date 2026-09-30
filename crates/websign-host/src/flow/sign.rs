//! The `sign.begin` flow (`docs/architecture/protocol.md` §Sign flow).
//!
//! ```text
//! Queued ──front──▶ Listing ──listed──▶ Selecting ──release──▶ AwaitingDigest(seq)
//!                                         ▲   │                  │  digest(seq)
//!                                         │   └──switch cert─────┤
//!                                         │                      ▼
//!                                         └──switch cert──── Ready(seq) ──Sign──▶ Signing
//!                                                                 ▲                 │
//!                                                  PinIncorrect ──┘   ok ──▶ Done(result)
//! any ──cancel / timeout / disconnect / error──▶ Done(error)
//! ```
//!
//! "Release" is automatic for a remembered caller (the preselected
//! certificate) and needs "Continue" otherwise (`docs/plan.md` D11).

use std::time::Instant;

use websign_core::Fingerprint;
use websign_protocol::ErrorCode;
use websign_protocol::messages::{SignBegin, SignDigest};
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::RequestKey;

use super::Effect;
use crate::ports::{KeyReply, KeySnapshot};

/// Where a sign request is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignState {
    Queued,
    Listing,
    Selecting { selected: Option<Fingerprint> },
    AwaitingDigest { seq: u32, fingerprint: Fingerprint },
    Ready { seq: u32, fingerprint: Fingerprint },
    Signing { tag: u64, fingerprint: Fingerprint },
    Done,
}

/// One `sign.begin` request.
#[derive(Debug)]
pub struct SignFlow {
    pub key: RequestKey,
    pub request: SignBegin,
    pub remembered: bool,
    pub state: SignState,
    /// When the person must have decided (`limits::DECISION_TIMEOUT` after
    /// the request reached the screen).
    pub deadline: Option<Instant>,
}

impl SignFlow {
    /// A queued request.
    pub fn new(key: RequestKey, request: SignBegin, remembered: bool) -> SignFlow {
        SignFlow {
            key,
            request,
            remembered,
            state: SignState::Queued,
            deadline: None,
        }
    }

    /// The request reached the front of the queue.
    pub fn activate(&mut self, now: Instant) -> Vec<Effect> {
        let _ = now;
        todo!("SPEC.md §4")
    }

    /// A listing arrived (first one or a refresh after a device event).
    pub fn on_listed(&mut self, snapshot: &KeySnapshot) -> Vec<Effect> {
        let _ = snapshot;
        todo!("SPEC.md §4")
    }

    /// The person acted in the window.
    pub fn on_ui(&mut self, event: UiEvent) -> Vec<Effect> {
        let _ = event;
        todo!("SPEC.md §4")
    }

    /// The caller sent `sign.digest`. Stale `seq` → ignored; wrong length →
    /// `InvalidRequest` ends the request.
    pub fn on_digest(&mut self, digest: SignDigest) -> Vec<Effect> {
        let _ = digest;
        todo!("SPEC.md §4")
    }

    /// The key store answered.
    pub fn on_keys(&mut self, reply: &KeyReply) -> Vec<Effect> {
        let _ = reply;
        todo!("SPEC.md §4")
    }

    /// Ends the request with `code` (cancel, abort, timeout, disconnect).
    pub fn end(&mut self, code: ErrorCode) -> Vec<Effect> {
        let _ = code;
        todo!("SPEC.md §4")
    }
}
