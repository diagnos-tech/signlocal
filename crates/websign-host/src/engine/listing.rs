//! What the engine remembers of the key store listings: whether one is
//! running (for "Still reading {device}", `docs/ux.md` §4.8) and the last
//! one, whose certificate bodies "View in system" shows (§5.12).

use std::sync::Arc;

use websign_core::Fingerprint;
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::RequestKey;

use super::Engine;
use crate::ports::KeySnapshot;

/// The listings of this process.
#[derive(Debug, Default)]
pub(super) struct Listings {
    /// `List` commands sent and not answered yet (the worker answers each,
    /// in order).
    running: u32,
    /// The last listing: what the window lists now.
    last: Option<Arc<KeySnapshot>>,
}

impl Listings {
    pub fn started(&mut self) {
        self.running = self.running.saturating_add(1);
    }

    pub fn arrived(&mut self, snapshot: &Arc<KeySnapshot>) {
        self.running = self.running.saturating_sub(1);
        self.last = Some(Arc::clone(snapshot));
    }
}

impl Engine {
    /// A listing is taking long: the request on screen says which device is
    /// still being read. A notice that crossed its listing on the way (no
    /// listing runs any more) is stale; the `Certificates` that followed
    /// already took the window out of "Looking for certificates…".
    pub(super) fn on_slow_listing(&mut self, device: Option<String>) {
        if self.listings.running == 0 {
            return;
        }
        if let Some(key) = self.queue.active() {
            self.ports
                .ui
                .command(UiCommand::SlowListing { key, device });
        }
    }

    /// "View in system": the OS viewer gets the certificate's DER from the
    /// last listing. The window never holds certificate bodies.
    pub(super) fn view_certificate(&mut self, key: RequestKey, fingerprint: &Fingerprint) {
        if self.queue.active() != Some(key) {
            return;
        }
        let der = self
            .listings
            .last
            .as_ref()
            .and_then(|snapshot| snapshot.certificates.get(fingerprint));
        match der {
            Some(der) => {
                let der = der.clone();
                self.ports.ui.view_certificate(der);
            }
            None => log::debug!("the certificate to view is not listed"),
        }
    }
}
