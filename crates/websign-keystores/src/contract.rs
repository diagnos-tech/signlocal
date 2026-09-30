//! The contract every [`Keystore`] adapter must satisfy, as runnable checks.
//!
//! Written once, run by each adapter's integration test against software
//! keys on its OS (SoftHSM2 on Linux/macOS/Windows, CNG/CAPI software
//! providers on Windows, a temporary keychain on macOS). `SPEC.md` §7 lists
//! the checks and the fixtures each OS job prepares.

use crate::{Keystore, PinPrompt};

/// What the fixture on this machine guarantees.
#[derive(Debug, Clone)]
pub struct Fixture {
    /// SHA-256 fingerprints (lowercase hex) of certificates that must be listed.
    pub expected: Vec<String>,
    /// PIN for keys with [`PinPrompt::App`], from the CI environment.
    pub pin: Option<String>,
    /// A wrong PIN to exercise `WrongPin`; `None` skips that check (it
    /// spends a PIN attempt).
    pub wrong_pin: Option<String>,
}

/// One failed expectation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Stable check name, e.g. `"list-never-prompts"`.
    pub check: &'static str,
    pub detail: String,
}

/// Runs every contract check against `keystore`; empty = compliant.
pub fn run(keystore: &mut dyn Keystore, fixture: &Fixture) -> Vec<Violation> {
    let _ = (keystore, fixture, PinPrompt::System);
    todo!("SPEC.md §7")
}
