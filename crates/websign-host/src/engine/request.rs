//! One open request as the engine holds it.

use std::time::Instant;

use websign_protocol::{ErrorCode, RequestId};
use websign_ui_model::certs::ListContext;
use websign_ui_model::confirm::UiEvent;

use crate::caller::Caller;
use crate::flow::choose::{ChooseFlow, ChooseState};
use crate::flow::sign::{SignFlow, SignState};
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

    /// The digest deadline while a `sign.need_digest` is unanswered.
    pub fn track_digest(&mut self, now: Instant, wait: std::time::Duration) {
        let awaiting = match &self.flow {
            Flow::Sign(flow) => match flow.state {
                SignState::AwaitingDigest { seq, .. } => Some(seq),
                _ => None,
            },
            Flow::Choose(_) => None,
        };
        self.digest_wait = match (awaiting, self.digest_wait) {
            (Some(seq), Some((known, at))) if known == seq => Some((seq, at)),
            (Some(seq), _) => Some((seq, now + wait)),
            (None, _) => None,
        };
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
