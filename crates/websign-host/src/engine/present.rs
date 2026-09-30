//! What the engine tells a flow before it starts: who asks, and the inputs
//! of the certificate list rules.

use websign_core::Fingerprint;
use websign_core::present::caller::caller_label;
use websign_ui_model::certs::ListContext;
use websign_ui_model::confirm::port::{CallerView, RequestKey};

use super::Engine;
use super::persist::log_store;
use super::request::Flow;
use crate::caller::Caller;
use crate::flow::{Presentation, accepted_algorithms, list_context};

/// The window's view of a caller.
fn caller_view(caller: &Caller) -> CallerView {
    match caller {
        Caller::Web {
            origin,
            top,
            browser,
        } => CallerView::Web {
            origin: origin.clone(),
            top: top.clone(),
            browser: browser.name,
        },
        Caller::Desktop(program) => CallerView::Desktop {
            label: caller_label(program),
        },
    }
}

/// The fingerprints among `hex` that parse; a damaged entry is skipped.
pub(super) fn parse_fingerprints(hex: impl IntoIterator<Item = String>) -> Vec<Fingerprint> {
    hex.into_iter()
        .filter_map(|text| text.parse::<Fingerprint>().ok())
        .collect()
}

impl Engine {
    /// The presentation of request `key` at its current place in the queue.
    pub(super) fn presentation(&self, key: RequestKey) -> Option<Presentation> {
        let request = self.requests.get(&key)?;
        Some(Presentation {
            caller: caller_view(&request.caller),
            can_remember: request.caller.can_remember(),
            position: self.queue.position(),
        })
    }

    /// The list rules' inputs for request `key` now: this caller's last
    /// certificate and everyone's recent ones come from the stores, which are
    /// read fresh because other processes change them.
    pub(super) fn list_context_for(&mut self, key: RequestKey) -> Option<ListContext> {
        let request = self.requests.get(&key)?;
        let caller = request.caller.clone();
        let (accepted, requested) = match &request.flow {
            Flow::Sign(flow) => (
                flow.request.algorithms.clone().unwrap_or_default(),
                flow.request
                    .certificate
                    .as_ref()
                    .and_then(|hex| hex.as_str().parse::<Fingerprint>().ok()),
            ),
            Flow::Choose(flow) => (
                flow.request
                    .filter
                    .as_ref()
                    .and_then(|filter| filter.algorithms.clone())
                    .unwrap_or_default(),
                None,
            ),
        };
        let last_used_here = self
            .consent_of(&caller)
            .and_then(|record| parse_fingerprints(record.certificates).into_iter().next());
        let recent = match self.ports.stores.usage().recent() {
            Ok(recent) => parse_fingerprints(recent),
            Err(error) => {
                log_store("usage store unavailable", &error);
                Vec::new()
            }
        };
        Some(list_context(
            self.ports.clock.wall(),
            accepted_algorithms(&accepted),
            requested,
            last_used_here,
            recent,
        ))
    }
}
