//! Reading and writing the small stores. A failing store never fails a
//! request: the engine logs the kind of failure and carries on without it.

use websign_protocol::ErrorCode;
use websign_ui_model::confirm::port::RequestKey;

use super::Engine;
use crate::caller::Caller;
use crate::ports::unix_seconds;
use crate::store::{ConsentRecord, ErrorRecord, StoreError};

/// Logs a store failure by kind only: paths and details name the user's
/// profile folder.
pub(super) fn log_store(what: &str, error: &StoreError) {
    log::warn!("{what}: {}", error.kind());
}

impl Engine {
    /// The remembered record of `caller`, read fresh: another process or
    /// the diagnostics window may have revoked it a moment ago. A caller
    /// that cannot be remembered has none, even if an older version stored
    /// one (an interpreter remembered before it was recognized as one).
    pub(super) fn consent_of(&mut self, caller: &Caller) -> Option<ConsentRecord> {
        if !caller.can_remember() {
            return None;
        }
        match self.ports.stores.consent().get(&caller.consent_key()) {
            Ok(record) => record,
            Err(error) => {
                log_store("consent store unavailable", &error);
                None
            }
        }
    }

    /// A ticked "Remember" adds this certificate to the caller's consent
    /// (where allowed); otherwise the use only refreshes a certificate the
    /// consent already covers: using a certificate never grants consent.
    pub(super) fn record_consent(&mut self, key: RequestKey, remember: bool, fingerprint: &str) {
        let Some(request) = self.requests.get(&key) else {
            return;
        };
        let consent_key = request.caller.consent_key();
        let can_remember = request.caller.can_remember();
        let now = unix_seconds(self.ports.clock.wall());
        let consent = self.ports.stores.consent();
        let result = match (can_remember, remember) {
            (false, _) => Ok(()),
            (true, true) => consent.remember(&consent_key, fingerprint, now),
            (true, false) => consent.record_use(&consent_key, fingerprint, now),
        };
        if let Err(error) = result {
            log_store("consent not saved", &error);
        }
        if let Err(error) = self.ports.stores.usage().record(fingerprint, now) {
            log_store("usage not saved", &error);
        }
    }

    /// One line in the recent errors: codes and native status names only.
    pub(super) fn record_error(
        &mut self,
        key: RequestKey,
        code: ErrorCode,
        native: Option<String>,
    ) {
        let operation = self
            .requests
            .get(&key)
            .map_or("sign", |request| request.operation());
        let record = ErrorRecord {
            at: unix_seconds(self.ports.clock.wall()),
            operation: operation.to_owned(),
            code: code.as_str().to_owned(),
            source: source_of(native.as_deref()).to_owned(),
            native,
        };
        if let Err(error) = self.ports.stores.errors().record(record) {
            log_store("error not saved", &error);
        }
    }
}

/// Which layer produced a native status, from the name's prefix.
fn source_of(native: Option<&str>) -> &'static str {
    let Some(native) = native else {
        return "host";
    };
    if native.starts_with("CKR_") {
        "pkcs11"
    } else if native.starts_with("NTE_") || native.starts_with("SCARD_") {
        "windows"
    } else if native.starts_with("errSec") {
        "macos"
    } else {
        "host"
    }
}
