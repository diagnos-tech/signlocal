//! Building sign flows in a given state, driving them only through their
//! public methods.

use std::time::{Instant, SystemTime};

use websign_core::present::caller::CallerLabel;
use websign_core::{Fingerprint, HashAlgorithm};
use websign_host::flow::Effect;
use websign_host::flow::sign::SignFlow;
use websign_host::flow::{Presentation, list_context};
use websign_host::ports::{KeyCommand, KeyReply};
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
                runs_scripts: false,
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

/// The consent of a remembered caller that already used every fixture
/// certificate; `None` for a new caller.
pub fn consent(remembered: bool) -> Option<Vec<Fingerprint>> {
    remembered.then(|| {
        [Cert::p256(), Cert::rsa(), Cert::rsa_b(), Cert::person()]
            .iter()
            .map(|cert| cert.fingerprint)
            .collect()
    })
}

pub fn new_flow(remembered: bool) -> SignFlow {
    SignFlow::new(KEY, begin(HashName::Sha256), consent(remembered))
}

/// `effects` followed by what the flow does when the key store answers
/// every chain lookup among them with an empty chain: the real worker
/// serves commands in order, so a release completes this way.
pub fn with_chains(flow: &mut SignFlow, effects: Vec<Effect>) -> Vec<Effect> {
    let tags: Vec<u64> = effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Keys(KeyCommand::Chain { tag, .. }) => Some(*tag),
            _ => None,
        })
        .collect();
    let mut all = effects;
    for tag in tags {
        all.extend(flow.on_keys(&KeyReply::Chain {
            tag,
            chain: Vec::new(),
        }));
    }
    all
}

pub fn digest_message(seq: u32, bytes: Vec<u8>) -> SignDigest {
    SignDigest {
        seq,
        digest: Base64Bytes::new(bytes),
    }
}

/// A flow activated and listed with `certs`, its chain lookups answered.
pub fn listed(remembered: bool, certs: &[&Cert]) -> SignFlow {
    let mut flow = new_flow(remembered);
    flow.activate(Instant::now(), &presentation());
    let effects = flow.on_listed(&snapshot(certs), context());
    with_chains(&mut flow, effects);
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
pub fn press_sign(flow: &mut SignFlow, cert: &Cert) -> Vec<Effect> {
    flow.on_ui(UiEvent::Sign {
        key: KEY,
        fingerprint: cert.fingerprint,
        via: 0,
        pin: None,
        remember: false,
    })
}
