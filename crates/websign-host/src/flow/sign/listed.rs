//! A listing arrives.

use websign_ui_model::certs::ListContext;

use super::{SignFlow, SignState};
use crate::flow::Effect;
use crate::flow::listing::Listing;
use crate::ports::KeySnapshot;

impl SignFlow {
    /// A listing arrived (first one or a refresh after a device event).
    /// `context` holds the list rules' inputs now: the engine reads the
    /// caller's history and the request's preferences into it, so the
    /// preselection (`sign.begin.certificate`, then the certificate last used
    /// here, then the first usable) is the window's own.
    ///
    /// The first listing moves to `Selecting`, and for a remembered caller
    /// releases the preselected certificate at once. Later ones only refresh
    /// the window: the person's selection and any pending digest stay.
    pub fn on_listed(&mut self, snapshot: &KeySnapshot, context: ListContext) -> Vec<Effect> {
        if matches!(self.state, SignState::Queued | SignState::Done) {
            return Vec::new();
        }
        let listing = Listing::build(snapshot, context);
        let mut effects = vec![Effect::Ui(listing.command(self.key))];
        let preselected = listing.preselected();
        self.listing = Some(listing);
        if self.state == SignState::Listing {
            self.state = SignState::Selecting {
                selected: preselected,
            };
            if let (true, Some(fingerprint)) = (self.remembered, preselected) {
                effects.extend(self.release(fingerprint));
            }
        }
        effects
    }
}
