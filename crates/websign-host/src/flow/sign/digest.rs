//! The caller answers `sign.need_digest`.

use websign_protocol::messages::SignDigest;
use websign_protocol::verification_code;
use websign_ui_model::confirm::port::UiCommand;

use super::{SignFlow, SignState};
use crate::flow::Effect;

impl SignFlow {
    /// The caller sent `sign.digest`. Stale `seq` → ignored; wrong length →
    /// `InvalidRequest` ends the request.
    ///
    /// A digest for a sequence number never issued is a broken client, not a
    /// late one, and ends the request the same way.
    pub fn on_digest(&mut self, digest: SignDigest) -> Vec<Effect> {
        if self.state == SignState::Done {
            return Vec::new();
        }
        if digest.seq > self.seq {
            return self.fail_request("the site answered a digest request that was never made");
        }
        let SignState::AwaitingDigest { seq, fingerprint } = self.state.clone() else {
            return Vec::new();
        };
        if digest.seq != seq {
            return Vec::new();
        }
        let bytes = digest.digest.into_bytes();
        if self.hash().check_digest(&bytes).is_err() {
            return self.fail_request("the site sent a digest of the wrong length");
        }
        let Some(code) = verification_code(&bytes) else {
            return self.fail_request("the site sent a digest of the wrong length");
        };
        self.digest = Some(bytes);
        self.state = SignState::Ready { seq, fingerprint };
        vec![Effect::Ui(UiCommand::DigestReady {
            key: self.key,
            fingerprint,
            code,
        })]
    }
}
