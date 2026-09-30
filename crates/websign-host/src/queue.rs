//! One request on screen, the rest waiting (`docs/ux.md` §4.11).
//!
//! Pure bookkeeping of [`RequestKey`]s; flows hold the data. At most
//! `limits::MAX_QUEUED_REQUESTS` wait behind the active one; one more is
//! refused with `Busy`. Windowless requests (`status`, remembered `choose`)
//! never enter the queue.

use std::collections::VecDeque;

use websign_protocol::limits::MAX_QUEUED_REQUESTS;
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
        if self.active.is_none() {
            self.active = Some(key);
            return Ok(true);
        }
        if self.waiting.len() >= MAX_QUEUED_REQUESTS {
            return Err(Busy);
        }
        self.waiting.push_back(key);
        Ok(false)
    }

    /// Removes `key` wherever it is. When it was active, the next waiting
    /// request becomes active and is returned.
    pub fn remove(&mut self, key: RequestKey) -> Option<RequestKey> {
        if self.active == Some(key) {
            self.active = self.waiting.pop_front();
            return self.active;
        }
        self.waiting.retain(|waiting| *waiting != key);
        None
    }

    /// The request on screen.
    pub fn active(&self) -> Option<RequestKey> {
        self.active
    }

    /// `(1-based position of the active request, total)`, for the eyebrow.
    pub fn position(&self) -> (u32, u32) {
        match self.active {
            Some(_) => (1, u32::try_from(1 + self.waiting.len()).unwrap_or(u32::MAX)),
            None => (0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: u64) -> RequestKey {
        RequestKey(n)
    }

    #[test]
    fn first_request_is_active_and_ten_more_wait() {
        let mut queue = RequestQueue::default();
        assert_eq!(queue.push(key(0)), Ok(true));
        for n in 1..=MAX_QUEUED_REQUESTS as u64 {
            assert_eq!(queue.push(key(n)), Ok(false));
        }
        assert_eq!(queue.push(key(99)), Err(Busy));
        assert_eq!(queue.position(), (1, 11));
    }

    #[test]
    fn finishing_the_active_request_promotes_the_oldest_waiting() {
        let mut queue = RequestQueue::default();
        for n in 0..3 {
            let _ = queue.push(key(n));
        }
        assert_eq!(queue.remove(key(0)), Some(key(1)));
        assert_eq!(queue.active(), Some(key(1)));
        assert_eq!(queue.position(), (1, 2));
    }

    #[test]
    fn removing_a_waiting_request_keeps_the_active_one() {
        let mut queue = RequestQueue::default();
        for n in 0..3 {
            let _ = queue.push(key(n));
        }
        assert_eq!(queue.remove(key(1)), None);
        assert_eq!(queue.active(), Some(key(0)));
        assert_eq!(queue.remove(key(0)), Some(key(2)));
        assert_eq!(queue.remove(key(2)), None);
        assert_eq!(queue.position(), (0, 0));
    }
}
