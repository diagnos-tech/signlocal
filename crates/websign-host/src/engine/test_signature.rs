//! The setup test: the first signature the project's own site gets marks
//! the diagnostics "You're ready to sign" card as done.

use websign_core::present::origin::format_origin;
use websign_ui_model::confirm::port::RequestKey;

use super::Engine;
use super::persist::log_store;
use crate::caller::Caller;

/// The canonical origin of the project's site: `HOMEPAGE` without its path.
pub(super) fn project_origin() -> Option<String> {
    let homepage = websign_project::HOMEPAGE;
    let (scheme, rest) = homepage.split_once("://")?;
    let host = rest.split('/').next()?;
    let origin = format_origin(&format!("{scheme}://{host}")).ok()?;
    Some(origin.canonical)
}

/// Whether a signature for `caller` proves the setup works: only the web
/// page the project itself serves for that purpose. Any other site, and any
/// desktop program, could be signing for its own reasons.
fn is_test_page(caller: &Caller) -> bool {
    let Caller::Web { origin, .. } = caller else {
        return false;
    };
    if project_origin().is_some_and(|site| site == origin.canonical) {
        return true;
    }
    local_test_origin(&origin.canonical)
}

/// End-to-end and unit-test runs serve the test page from `localhost`; a
/// release build never trusts it (any local program could listen there).
#[cfg(any(test, feature = "e2e"))]
fn local_test_origin(canonical: &str) -> bool {
    canonical == "http://localhost"
        || canonical
            .strip_prefix("http://localhost:")
            .is_some_and(|port| !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(not(any(test, feature = "e2e")))]
fn local_test_origin(_canonical: &str) -> bool {
    false
}

impl Engine {
    /// Records `test_signature_done` after a successful signature of the
    /// test page. Nothing else is stored: no origin, no certificate.
    pub(super) fn record_test_signature(&mut self, key: RequestKey) {
        let Some(request) = self.requests.get(&key) else {
            return;
        };
        if !is_test_page(&request.caller) {
            return;
        }
        let result = self
            .ports
            .stores
            .settings()
            .update(&mut |settings| settings.test_signature_done = true);
        if let Err(error) = result {
            log_store("test signature not saved", &error);
        }
    }
}
