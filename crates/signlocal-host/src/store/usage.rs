//! When each certificate was last used by anyone, for list order
//! (`docs/ux.md` §5.9 rule 2).

use super::StoreError;

/// Certificate fingerprints (SHA-256 hex) by most recent use.
pub trait UsageStore {
    /// Newest first.
    fn recent(&mut self) -> Result<Vec<String>, StoreError>;
    fn record(&mut self, fingerprint: &str, now: i64) -> Result<(), StoreError>;
}
