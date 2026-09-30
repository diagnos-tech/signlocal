//! The `choose` flow: remembered caller → its certificates at once, no
//! window; otherwise the window in choose mode (`docs/plan.md` D2).

use std::time::Instant;

use websign_protocol::ErrorCode;
use websign_protocol::messages::Choose;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::RequestKey;

use super::Effect;
use crate::ports::KeySnapshot;

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
        }
    }

    /// Whether it can be answered without the window (and so skips the queue).
    pub fn answers_without_window(&self) -> bool {
        !self.remembered.is_empty()
    }

    /// The request reached the front of the queue (or runs windowless).
    pub fn activate(&mut self, now: Instant) -> Vec<Effect> {
        let _ = now;
        todo!("SPEC.md §5")
    }

    /// A listing arrived.
    pub fn on_listed(&mut self, snapshot: &KeySnapshot) -> Vec<Effect> {
        let _ = snapshot;
        todo!("SPEC.md §5")
    }

    /// The person acted in the window.
    pub fn on_ui(&mut self, event: UiEvent) -> Vec<Effect> {
        let _ = event;
        todo!("SPEC.md §5")
    }

    /// Ends the request with `code`.
    pub fn end(&mut self, code: ErrorCode) -> Vec<Effect> {
        let _ = code;
        todo!("SPEC.md §5")
    }
}
