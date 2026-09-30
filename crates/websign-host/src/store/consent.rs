//! Remembered callers (`docs/ux.md` §4.10): "Remember this site/program".

use serde::{Deserialize, Serialize};

use super::StoreError;

/// One remembered caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsentRecord {
    /// `Caller::consent_key`: canonical origin, `app:…` or `path:…`.
    pub key: String,
    /// Unix seconds.
    pub remembered_at: i64,
    pub last_used_at: i64,
    /// Certificates used with this caller (SHA-256 hex), most recent first.
    pub certificates: Vec<String>,
}

/// Consent, keyed by caller.
pub trait ConsentStore {
    fn get(&mut self, key: &str) -> Result<Option<ConsentRecord>, StoreError>;
    /// Every record, for Diagnostics › Browsers › Allowed sites/programs.
    fn list(&mut self) -> Result<Vec<ConsentRecord>, StoreError>;
    /// Creates or refreshes the record (ticked "Remember" and signed/chose).
    fn remember(&mut self, key: &str, fingerprint: &str, now: i64) -> Result<(), StoreError>;
    /// Moves `fingerprint` to the front and updates `last_used_at`, only if
    /// the caller is remembered.
    fn record_use(&mut self, key: &str, fingerprint: &str, now: i64) -> Result<(), StoreError>;
    fn revoke(&mut self, key: &str) -> Result<(), StoreError>;
}
