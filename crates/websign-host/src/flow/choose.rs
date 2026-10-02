//! The `choose` flow: remembered caller → its certificates at once, no
//! window; otherwise the window in choose mode (`docs/plan.md` D2).
//!
//! ```text
//! Queued ──activate──▶ Listing ──listed, remembered present──▶ (chains) ──▶ Done(result)
//!                         │ └──listed, none present──▶ Queued (as a new caller)
//!                         └──listed, windowed──▶ Choosing ──Choose──▶ (chain) ──▶ Done(result)
//! any ──cancel / timeout / disconnect──▶ Done(error)
//! ```
//!
//! Every answer carries the issuer chain of its certificates, so the flow
//! asks the key store for them and answers when the last one arrives.

mod answer;

use std::time::Instant;

use websign_protocol::ErrorCode;
use websign_protocol::limits::DECISION_TIMEOUT;
use websign_protocol::messages::Choose;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::{Finish, Mode, RequestKey, UiCommand};

use super::errors::{error_reply, finished};
use super::listing::Listing;
use super::{Effect, Presentation};
use crate::ports::KeyCommand;
use answer::Answer;

/// Where a choose request is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChooseState {
    Queued,
    Listing,
    Choosing,
    Done,
}

/// One `choose` request.
#[derive(Debug)]
pub struct ChooseFlow {
    pub key: RequestKey,
    pub request: Choose,
    /// Fingerprints the caller already used, when remembered.
    pub remembered: Vec<String>,
    pub state: ChooseState,
    pub deadline: Option<Instant>,
    listing: Option<Listing>,
    /// The window is (or will be) showing this request.
    windowed: bool,
    /// The reply being completed with issuer chains.
    answer: Option<Answer>,
    /// Counter of the key store operations this flow started.
    operations: u32,
}

impl ChooseFlow {
    /// A queued request.
    pub fn new(key: RequestKey, request: Choose, remembered: Vec<String>) -> ChooseFlow {
        ChooseFlow {
            key,
            request,
            remembered,
            state: ChooseState::Queued,
            deadline: None,
            listing: None,
            windowed: false,
            answer: None,
            operations: 0,
        }
    }

    /// Whether it can be answered without the window (and so skips the queue).
    pub fn answers_without_window(&self) -> bool {
        !self.remembered.is_empty()
    }

    /// The request reached the front of the queue (or runs windowless).
    /// `presentation` is who asks and where the request stands.
    pub fn activate(&mut self, now: Instant, presentation: &Presentation) -> Vec<Effect> {
        self.state = ChooseState::Listing;
        self.deadline = Some(now + DECISION_TIMEOUT);
        self.windowed = !self.answers_without_window();
        let mut effects = Vec::new();
        if self.windowed {
            let open = presentation.open(self.key, Mode::Choose, false, Vec::new());
            effects.push(Effect::Ui(UiCommand::Open(open)));
        }
        effects.push(Effect::Keys(KeyCommand::List { refresh: false }));
        effects
    }

    /// The person acted in the window.
    pub fn on_ui(&mut self, event: UiEvent) -> Vec<Effect> {
        match event {
            UiEvent::Choose {
                key,
                fingerprint,
                remember,
            } if key == self.key => self.chosen(fingerprint, remember),
            UiEvent::Cancel { key, code } if key == self.key => self.end(code),
            _ => Vec::new(),
        }
    }

    /// Ends the request with `code` (cancel, abort, timeout).
    pub fn end(&mut self, code: ErrorCode) -> Vec<Effect> {
        let shown = self.shown();
        if !self.finish_now() {
            return Vec::new();
        }
        let mut effects = vec![error_reply(code)];
        if shown {
            effects.push(finished(self.key, code));
        }
        effects
    }

    /// The client is gone: nothing can be sent, and the window says the site
    /// cancelled.
    pub fn disconnected(&mut self) -> Vec<Effect> {
        let shown = self.shown();
        if !self.finish_now() || !shown {
            return Vec::new();
        }
        vec![Effect::Ui(UiCommand::Finished {
            key: self.key,
            finish: Finish::SiteCancelled,
        })]
    }

    /// Whether the window has been told about this request.
    fn shown(&self) -> bool {
        self.windowed && !matches!(self.state, ChooseState::Queued | ChooseState::Done)
    }

    fn finish_now(&mut self) -> bool {
        if self.state == ChooseState::Done {
            return false;
        }
        self.state = ChooseState::Done;
        self.answer = None;
        true
    }

    fn next_tag(&mut self) -> u64 {
        self.operations += 1;
        super::operation_tag(self.key, self.operations)
    }
}
