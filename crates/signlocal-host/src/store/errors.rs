//! The last errors, for "Copy diagnostics" (`docs/ux.md` §8.7).

use serde::{Deserialize, Serialize};

use super::StoreError;

/// How many are kept.
pub const MAX_RECENT_ERRORS: usize = 20;

/// One error: codes only, never data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRecord {
    /// Unix seconds.
    pub at: i64,
    /// `"sign"`, `"choose"`, `"list"`, `"hello"`.
    pub operation: String,
    /// Wire error code.
    pub code: String,
    /// `"windows"`, `"macos"`, `"pkcs11"`, `"host"`.
    pub source: String,
    /// Native status name, e.g. `"CKR_PIN_INCORRECT"`.
    pub native: Option<String>,
}

/// A ring of the newest [`MAX_RECENT_ERRORS`] errors.
pub trait ErrorStore {
    fn record(&mut self, record: ErrorRecord) -> Result<(), StoreError>;
    fn list(&mut self) -> Result<Vec<ErrorRecord>, StoreError>;
}
