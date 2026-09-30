//! Extension pre-registration on Windows: `HKCU\Software\Google\Chrome\
//! Extensions\<id>` (and the Edge/Brave equivalents) with the store's
//! `update_url`, so the browser offers "New extension added — Enable" on its
//! next start. Only for extensions already published in that store; a no-op
//! elsewhere.

use crate::destination::Outcome;

/// Writes (or, with `remove`, deletes) the pre-registration keys for every
/// published store ID.
pub fn apply(remove: bool, dry_run: bool) -> Vec<(String, Outcome)> {
    let _ = (remove, dry_run);
    todo!("SPEC.md §5")
}
