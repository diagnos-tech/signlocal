//! Building sign flows in a given state, driving them only through their
//! public methods.

use std::time::{Instant, SystemTime};

use websign_core::HashAlgorithm;
use websign_core::present::caller::CallerLabel;
use websign_host::flow::sign::SignFlow;
use websign_host::flow::{Presentation, list_context};
use websign_protocol::messages::{SignBegin, SignDigest};
use websign_protocol::types::{Base64Bytes, HashName};
use websign_ui_model::certs::ListContext;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::{CallerView, RequestKey};

use super::certs::{Cert, digest, snapshot};

pub const KEY: RequestKey = RequestKey(1);

/// A desktop program alone in the queue: flows only pass it to the window.
pub fn presentation() -> Presentation {
    Presentation {
        caller: CallerView::Desktop {
            label: CallerLabel {
                name: "Invoicer".to_owned(),
                detail: "/opt/tools/invoicer".to_owned(),
                verified: false,
            },
        },
        can_remember: true,
        position: (1, 1),
    }
}

/// The list rules' inputs of a caller without history that accepts any
/// algorithm; the engine builds the real one from the stores.
pub fn context() -> ListContext {
    list_context(SystemTime::now(), Vec::new(), None, None, Vec::new())
}

pub fn begin(hash: HashName) -> SignBegin {
    SignBegin {
        web: None,
        hash,
        algorithms: None,
        certificate: None,
    }
}

pub fn new_flow(remembered: bool) -> SignFlow {
    SignFlow::new(KEY, begin(HashName::Sha256), remembered)
}

pub fn digest_message(seq: u32, bytes: Vec<u8>) -> SignDigest {
    SignDigest {
        seq,
        digest: Base64Bytes::new(bytes),
    }
}

/// A flow activated and listed with `certs`.
pub fn listed(remembered: bool, certs: &[&Cert]) -> SignFlow {
    let mut flow = new_flow(remembered);
    flow.activate(Instant::now(), &presentation());
    flow.on_listed(&snapshot(certs), context());
    flow
}

/// A flow in `AwaitingDigest(1)` for `cert`.
pub fn awaiting(cert: &Cert) -> SignFlow {
    listed(true, &[cert])
}

/// A flow in `Ready(1)` for `cert` with the SHA-256 fixture digest.
pub fn ready(cert: &Cert) -> SignFlow {
    let mut flow = awaiting(cert);
    flow.on_digest(digest_message(1, digest(HashAlgorithm::Sha256)));
    flow
}

/// The window presses Sign for `cert` (path 0, no PIN).
pub fn press_sign(flow: &mut SignFlow, cert: &Cert) -> Vec<websign_host::flow::Effect> {
    flow.on_ui(UiEvent::Sign {
        key: KEY,
        fingerprint: cert.fingerprint,
        via: 0,
        pin: None,
        remember: false,
    })
}
