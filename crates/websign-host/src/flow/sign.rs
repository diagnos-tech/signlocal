//! The `sign.begin` flow (`docs/architecture/protocol.md` §Sign flow).
//!
//! ```text
//! Queued ──front──▶ Listing ──listed──▶ Selecting ──release──▶ [Releasing] ──▶ AwaitingDigest(seq)
//!                                         ▲   │                                   │  digest(seq)
//!                                         │   └──switch cert──────────────────────┤
//!                                         │                                       ▼
//!                                         └──switch cert──────────────────── Ready(seq) ──Sign──▶ Signing
//!                                                                                  ▲                 │
//!                                                                   PinIncorrect ──┘   ok ──▶ Done(result)
//! any ──cancel / timeout / disconnect / error──▶ Done(error)
//! ```
//!
//! "Release" is automatic only for a certificate in the caller's consent
//! record, and needs "Continue" for any other one, even from a remembered
//! caller (`docs/plan.md` D11): on a shared computer, a site one person
//! remembered must never see another person's certificate unasked.
//! `Releasing` waits (at most [`CHAIN_WAIT`]) for the certificate's issuer
//! chain, so `sign.need_digest` carries it.

mod deadlines;
mod digest;
mod failures;
mod keys;
mod listed;
mod release;
mod signing;
mod ui;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use websign_core::Fingerprint;
use websign_protocol::ErrorCode;
use websign_protocol::limits::DECISION_TIMEOUT;
use websign_protocol::messages::SignBegin;
use websign_ui_model::confirm::port::{Finish, Mode, RequestKey, UiCommand};

use super::errors::{error_reply, finished};
use super::listing::Listing;
use super::{Effect, Presentation};
use crate::ports::KeyCommand;

/// How long a release waits for the issuer chain before `sign.need_digest`
/// goes without it: key stores answer from memory, but a slow driver must
/// not hold the site's `prepare` back.
pub const CHAIN_WAIT: Duration = Duration::from_secs(2);

/// Where a sign request is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignState {
    Queued,
    Listing,
    Selecting {
        selected: Option<Fingerprint>,
    },
    /// Released: the window shows "Preparing…" while the issuer chain
    /// lookup `tag` runs; `sign.need_digest` follows it.
    Releasing {
        tag: u64,
        fingerprint: Fingerprint,
    },
    AwaitingDigest {
        seq: u32,
        fingerprint: Fingerprint,
    },
    Ready {
        seq: u32,
        fingerprint: Fingerprint,
    },
    Signing {
        tag: u64,
        fingerprint: Fingerprint,
    },
    Done,
}

/// One `sign.begin` request.
#[derive(Debug)]
pub struct SignFlow {
    pub key: RequestKey,
    pub request: SignBegin,
    /// The caller has a consent record (the chip says "Allowed site").
    pub remembered: bool,
    /// The certificates that record covers: the only ones released without
    /// "Continue".
    pub consented: Vec<Fingerprint>,
    pub state: SignState,
    /// When the person must have decided (`limits::DECISION_TIMEOUT` after
    /// the request reached the screen). See [`SignFlow::deadline_now`].
    pub deadline: Option<Instant>,
    listing: Option<Listing>,
    /// Sequence number of the last `sign.need_digest`.
    seq: u32,
    /// Counter of the key store operations this flow started.
    operations: u32,
    /// The digest for the current `Ready` state.
    digest: Option<Vec<u8>>,
    /// The signing in progress or last attempted.
    attempt: Option<signing::Attempt>,
    /// "Remember" as ticked in the window when the person signed.
    remember_choice: bool,
    chains: HashMap<Fingerprint, Vec<Vec<u8>>>,
    /// Chain lookups in flight, by tag.
    chain_lookups: HashMap<u64, Fingerprint>,
}

impl SignFlow {
    /// A queued request. `consented` is the caller's consent record: `None`
    /// when it is not remembered, else the certificates it covers.
    pub fn new(
        key: RequestKey,
        request: SignBegin,
        consented: Option<Vec<Fingerprint>>,
    ) -> SignFlow {
        SignFlow {
            key,
            request,
            remembered: consented.is_some(),
            consented: consented.unwrap_or_default(),
            state: SignState::Queued,
            deadline: None,
            listing: None,
            seq: 0,
            operations: 0,
            digest: None,
            attempt: None,
            remember_choice: false,
            chains: HashMap::new(),
            chain_lookups: HashMap::new(),
        }
    }

    /// The request reached the front of the queue. `presentation` is who
    /// asks and where the request stands, which only the engine knows.
    pub fn activate(&mut self, now: Instant, presentation: &Presentation) -> Vec<Effect> {
        self.state = SignState::Listing;
        self.deadline = Some(now + DECISION_TIMEOUT);
        let mode = Mode::Sign {
            hash: self.request.hash,
        };
        vec![
            Effect::Ui(UiCommand::Open(presentation.open(
                self.key,
                mode,
                self.remembered,
                self.consented.clone(),
            ))),
            Effect::Keys(KeyCommand::List { refresh: false }),
        ]
    }

    /// Ends the request with `code` (cancel, abort, timeout, disconnect).
    pub fn end(&mut self, code: ErrorCode) -> Vec<Effect> {
        let on_screen = self.on_screen();
        if !self.finish_now() {
            return Vec::new();
        }
        let mut effects = vec![error_reply(code)];
        if on_screen {
            effects.push(finished(self.key, code));
        }
        effects
    }

    /// The client is gone: nothing can be sent, and the window says the site
    /// cancelled.
    pub fn disconnected(&mut self) -> Vec<Effect> {
        let on_screen = self.on_screen();
        if !self.finish_now() || !on_screen {
            return Vec::new();
        }
        vec![Effect::Ui(UiCommand::Finished {
            key: self.key,
            finish: Finish::SiteCancelled,
        })]
    }

    /// Whether the window has shown this request (it left the queue).
    fn on_screen(&self) -> bool {
        !matches!(self.state, SignState::Queued | SignState::Done)
    }

    /// Moves to `Done`; `false` when it already was.
    fn finish_now(&mut self) -> bool {
        if self.state == SignState::Done {
            return false;
        }
        self.state = SignState::Done;
        self.digest = None;
        true
    }

    /// Whether the caller's consent covers `fingerprint`.
    fn is_consented(&self, fingerprint: &Fingerprint) -> bool {
        self.consented.contains(fingerprint)
    }

    fn next_tag(&mut self) -> u64 {
        self.operations += 1;
        super::operation_tag(self.key, self.operations)
    }
}
