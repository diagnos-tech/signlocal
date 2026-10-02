//! One open request as the engine holds it.

use std::time::Instant;

use websign_protocol::limits::DIGEST_TIMEOUT;
use websign_protocol::{ErrorCode, RequestId};
use websign_ui_model::certs::ListContext;
use websign_ui_model::confirm::UiEvent;

use crate::caller::Caller;
use crate::flow::choose::{ChooseFlow, ChooseState};
use crate::flow::sign::{CHAIN_WAIT, SignFlow, SignState};
use crate::flow::{Effect, Presentation};
use crate::ports::{KeyReply, KeySnapshot};

/// The flow of a request, whichever kind it is.
#[derive(Debug)]
pub(super) enum Flow {
    Sign(Box<SignFlow>),
    Choose(Box<ChooseFlow>),
}

/// A request between its first frame and its final reply.
#[derive(Debug)]
pub(super) struct Request {
    pub id: RequestId,
    pub caller: Caller,
    pub flow: Flow,
    /// Whether it took a place in the queue (windowless `choose` does not,
    /// unless it turns out to need the window).
    pub queued: bool,
    /// The digest the caller owes, by `seq` and deadline.
    pub digest_wait: Option<(u32, Instant)>,
    /// The issuer chain a release waits for, by lookup tag and deadline.
    pub chain_wait: Option<(u64, Instant)>,
}

impl Request {
    pub fn is_done(&self) -> bool {
        match &self.flow {
            Flow::Sign(flow) => flow.state == SignState::Done,
            Flow::Choose(flow) => flow.state == ChooseState::Done,
        }
    }

    /// Kind for the log and the errors store.
    pub fn operation(&self) -> &'static str {
        match self.flow {
            Flow::Sign(_) => "sign",
            Flow::Choose(_) => "choose",
        }
    }

    /// The decision deadline that applies now.
    pub fn deadline(&self) -> Option<Instant> {
        match &self.flow {
            Flow::Sign(flow) => flow.deadline_now(),
            Flow::Choose(flow) => flow.deadline,
        }
    }

    /// The digest deadline while a `sign.need_digest` is unanswered, and
    /// the chain deadline while a release waits for its chain.
    pub fn track_waits(&mut self, now: Instant) {
        let (awaiting, releasing) = match &self.flow {
            Flow::Sign(flow) => match flow.state {
                SignState::AwaitingDigest { seq, .. } => (Some(seq), None),
                SignState::Releasing { tag, .. } => (None, Some(tag)),
                _ => (None, None),
            },
            Flow::Choose(_) => (None, None),
        };
        self.digest_wait = keep_or_start(awaiting, self.digest_wait, now + DIGEST_TIMEOUT);
        self.chain_wait = keep_or_start(releasing, self.chain_wait, now + CHAIN_WAIT);
    }

    /// The effects of the waits that ran out at `now`: a chain that took too
    /// long is sent without, a digest that took too long ends the request.
    pub fn expire_waits(&mut self, now: Instant) -> Vec<Effect> {
        let Flow::Sign(flow) = &mut self.flow else {
            return Vec::new();
        };
        if self.digest_wait.is_some_and(|(_, at)| now >= at) {
            return flow.digest_timed_out();
        }
        if self.chain_wait.is_some_and(|(_, at)| now >= at) {
            return flow.chain_wait_over();
        }
        Vec::new()
    }

    /// Whether a wait of [`Request::track_waits`] ran out at `now`.
    pub fn wait_expired(&self, now: Instant) -> bool {
        self.digest_wait.is_some_and(|(_, at)| now >= at)
            || self.chain_wait.is_some_and(|(_, at)| now >= at)
    }

    pub fn activate(&mut self, now: Instant, presentation: &Presentation) -> Vec<Effect> {
        match &mut self.flow {
            Flow::Sign(flow) => flow.activate(now, presentation),
            Flow::Choose(flow) => flow.activate(now, presentation),
        }
    }

    pub fn on_listed(&mut self, snapshot: &KeySnapshot, context: ListContext) -> Vec<Effect> {
        match &mut self.flow {
            Flow::Sign(flow) => flow.on_listed(snapshot, context),
            Flow::Choose(flow) => flow.on_listed(snapshot, context),
        }
    }

    pub fn on_ui(&mut self, event: UiEvent) -> Vec<Effect> {
        match &mut self.flow {
            Flow::Sign(flow) => flow.on_ui(event),
            Flow::Choose(flow) => flow.on_ui(event),
        }
    }

    pub fn on_keys(&mut self, reply: &KeyReply) -> Vec<Effect> {
        match &mut self.flow {
            Flow::Sign(flow) => flow.on_keys(reply),
            Flow::Choose(flow) => flow.on_keys(reply),
        }
    }

    pub fn end(&mut self, code: ErrorCode) -> Vec<Effect> {
        match &mut self.flow {
            Flow::Sign(flow) => flow.end(code),
            Flow::Choose(flow) => flow.end(code),
        }
    }

    pub fn disconnected(&mut self) -> Vec<Effect> {
        match &mut self.flow {
            Flow::Sign(flow) => flow.disconnected(),
            Flow::Choose(flow) => flow.disconnected(),
        }
    }

    /// A `choose` that found nothing remembered and now needs the window.
    pub fn needs_window(&self) -> bool {
        !self.queued
            && matches!(&self.flow, Flow::Choose(flow) if flow.state == ChooseState::Queued)
    }
}

/// The deadline of the wait for `current`: kept while it is the same wait,
/// started at `start` for a new one, cleared when nothing is awaited.
fn keep_or_start<T: PartialEq + Copy>(
    current: Option<T>,
    known: Option<(T, Instant)>,
    start: Instant,
) -> Option<(T, Instant)> {
    match (current, known) {
        (Some(id), Some((was, at))) if was == id => Some((id, at)),
        (Some(id), _) => Some((id, start)),
        (None, _) => None,
    }
}
