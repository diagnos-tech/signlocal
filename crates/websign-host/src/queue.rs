//! One request on screen, the rest waiting (`docs/ux.md` §4.11).
//!
//! Pure bookkeeping of [`RequestKey`]s; flows hold the data. At most
//! `limits::MAX_QUEUED_REQUESTS` wait behind the active one; one more is
//! refused with `Busy`. Windowless requests (`status`, remembered `choose`)
//! never enter the queue.

use std::collections::VecDeque;

use websign_ui_model::confirm::port::RequestKey;

/// The queue is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("too many requests are waiting")]
pub struct Busy;

/// Active request plus the waiting line.
#[derive(Debug, Default)]
pub struct RequestQueue {
    active: Option<RequestKey>,
    waiting: VecDeque<RequestKey>,
}

impl RequestQueue {
    /// Adds `key`. `Ok(true)` = it became active now.
    pub fn push(&mut self, key: RequestKey) -> Result<bool, Busy> {
        let _ = key;
        todo!("SPEC.md §6")
    }

    /// Removes `key` wherever it is. When it was active, the next waiting
    /// request becomes active and is returned.
    pub fn remove(&mut self, key: RequestKey) -> Option<RequestKey> {
        let _ = key;
        todo!("SPEC.md §6")
    }

    /// The request on screen.
    pub fn active(&self) -> Option<RequestKey> {
        self.active
    }

    /// `(1-based position of the active request, total)`, for the eyebrow.
    pub fn position(&self) -> (u32, u32) {
        let _ = &self.waiting;
        todo!("SPEC.md §6")
    }
}
