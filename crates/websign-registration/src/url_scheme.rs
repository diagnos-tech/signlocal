//! The `websign:` URL scheme, so the website's `/activate` page can start the
//! app once to register it with browsers (stores run no install scripts).
//!
//! Direct builds register it per user: `HKCU\Software\Classes\websign`
//! (Windows), a `.desktop` file with `x-scheme-handler/websign` (Linux). On
//! macOS the `.app`'s `CFBundleURLTypes` declares it; nothing to write.
//! Packaged builds declare it in their manifests.

use std::path::Path;

use crate::destination::Outcome;

/// Registers `executable` as the handler of `websign:` URLs.
pub fn register(executable: &Path, dry_run: bool) -> Outcome {
    let _ = (executable, dry_run);
    todo!("SPEC.md §4")
}

/// Removes the per-user handler registration.
pub fn unregister(dry_run: bool) -> Outcome {
    let _ = dry_run;
    todo!("SPEC.md §4")
}
