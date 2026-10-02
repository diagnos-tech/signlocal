//! One process-wide lock around every Security.framework call.
//!
//! The file-based keychain code behind `SecItemCopyMatching` and `SecKey` on
//! macOS is not thread-safe; Chromium puts every call behind one lock for
//! that reason (`crypto/mac_security_services_lock.h`). The host already
//! confines the hub to one thread, so the lock is normally uncontended; it
//! keeps the rule true if a second hub or a diagnostics thread ever calls in.
//!
//! The lock is held while the OS shows its PIN dialog inside
//! `SecKeyCreateSignature`. That is intended: a listing started meanwhile
//! would race the token session the dialog is unlocking.

use std::sync::{Mutex, PoisonError};

static SECURITY_FRAMEWORK: Mutex<()> = Mutex::new(());

/// Runs `call` while holding the Security.framework lock.
///
/// Not reentrant: `call` must not call `serialized` again, so only the
/// [`crate::Keystore`] methods in `mod.rs` take it.
pub fn serialized<T>(call: impl FnOnce() -> T) -> T {
    // The lock guards no data, only exclusivity, so a panic in an earlier
    // holder leaves nothing inconsistent: recover from the poison.
    let _guard = SECURITY_FRAMEWORK
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    call()
}
