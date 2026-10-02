//! Putting requests on the screen, one at a time (`docs/ux.md` §4.11).

use websign_protocol::{ErrorCode, RequestId};
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::RequestKey;

use super::Engine;
use super::request::{Flow, Request};
use crate::caller::Caller;
use crate::queue::Busy;

impl Engine {
    /// Registers a new request and queues it. `Busy` answers the caller
    /// straight away: nothing was created.
    pub(super) fn enqueue(&mut self, key: RequestKey, id: RequestId, caller: Caller, flow: Flow) {
        match self.queue.push(key) {
            Err(Busy) => self.send_error(&id, ErrorCode::Busy, "too many requests are waiting"),
            Ok(active) => {
                self.session.open(id.clone(), key);
                self.requests.insert(
                    key,
                    Request {
                        id,
                        caller,
                        flow,
                        queued: true,
                        digest_wait: None,
                        chain_wait: None,
                    },
                );
                if active {
                    self.activate(key);
                } else {
                    self.refresh_position();
                }
            }
        }
    }

    /// Registers a request that never uses the queue and starts it at once.
    pub(super) fn start_windowless(
        &mut self,
        key: RequestKey,
        id: RequestId,
        caller: Caller,
        flow: Flow,
    ) {
        self.session.open(id.clone(), key);
        self.requests.insert(
            key,
            Request {
                id,
                caller,
                flow,
                queued: false,
                digest_wait: None,
                chain_wait: None,
            },
        );
        self.activate(key);
    }

    /// A key no other request of this process has.
    pub(super) fn new_key(&mut self) -> RequestKey {
        let key = RequestKey(self.next_key);
        self.next_key += 1;
        key
    }

    /// The request reached the front of the queue (or runs windowless):
    /// tell its flow who asks and where it stands, then start it.
    pub(super) fn activate(&mut self, key: RequestKey) {
        let Some(presentation) = self.presentation(key) else {
            return;
        };
        let now = self.ports.clock.now();
        let Some(request) = self.requests.get_mut(&key) else {
            return;
        };
        let effects = request.activate(now, &presentation);
        self.perform(key, effects);
    }

    /// The window shows "request 1 of 3": tell it when the count changes.
    pub(super) fn refresh_position(&mut self) {
        if let Some(key) = self.queue.active() {
            self.ports.ui.command(UiCommand::Queue {
                key,
                position: self.queue.position(),
            });
        }
    }

    /// A windowless `choose` whose remembered certificates are gone waits
    /// for the window like any new caller.
    pub(super) fn queue_for_window(&mut self, key: RequestKey) {
        match self.queue.push(key) {
            Ok(true) => {
                self.mark_queued(key);
                self.activate(key);
            }
            Ok(false) => {
                self.mark_queued(key);
                self.refresh_position();
            }
            Err(Busy) => {
                let effects = self
                    .requests
                    .get_mut(&key)
                    .map(|request| request.end(ErrorCode::Busy))
                    .unwrap_or_default();
                self.perform(key, effects);
            }
        }
    }

    fn mark_queued(&mut self, key: RequestKey) {
        if let Some(request) = self.requests.get_mut(&key) {
            request.queued = true;
        }
    }
}
