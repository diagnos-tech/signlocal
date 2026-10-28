//! The contract every [`Keystore`] adapter must satisfy, as runnable checks.
//!
//! Written once, run by each adapter's integration test against software
//! keys on its OS (SoftHSM2 on Linux/macOS/Windows, CNG/CAPI software
//! providers on Windows, a temporary keychain on macOS). `SPEC.md` §7 lists
//! the checks and the fixtures each OS job prepares.
//!
//! The suite only signs with the fixture's keys (`Fixture::expected`): a
//! developer machine may also hold real cards, which must not be touched.

mod listing;
mod pin;
mod signing;
mod support;

use crate::{FoundKey, Keystore};

/// What the fixture on this machine guarantees.
#[derive(Debug, Clone, Default)]
pub struct Fixture {
    /// SHA-256 fingerprints (lowercase hex) of certificates that must be listed.
    pub expected: Vec<String>,
    /// PIN for keys with [`PinPrompt::App`](crate::PinPrompt::App), from the
    /// CI environment.
    pub pin: Option<String>,
    /// A wrong PIN to exercise `WrongPin`; `None` skips that check (it
    /// spends a PIN attempt).
    pub wrong_pin: Option<String>,
    /// Text that must never appear in `provider`: token labels, serial
    /// numbers, key container names of the fixture. Holder names and
    /// certificate serials are taken from the certificates themselves.
    pub private_text: Vec<String>,
}

/// One failed expectation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Stable check name, e.g. `"list-is-quiet"`.
    pub check: &'static str,
    pub detail: String,
}

/// Runs every contract check against `keystore`; empty = compliant.
pub fn run(keystore: &mut dyn Keystore, fixture: &Fixture) -> Vec<Violation> {
    let mut report = Report::default();
    keystore.end_sessions();
    let targets = listing::check(keystore, fixture, &mut report);
    for key in &targets {
        signing::check(keystore, key, fixture, &mut report);
    }
    if let Some(key) = targets.iter().find(|key| pin::app_collects_pin(key)) {
        pin::check_wrong_pin(keystore, key, fixture, &mut report);
    }
    for key in targets.iter().filter(|key| pin::app_collects_pin(key)) {
        pin::check_sessions(keystore, key, fixture, &mut report);
    }
    keystore.end_sessions();
    report.violations
}

/// Violations collected so far.
#[derive(Debug, Default)]
struct Report {
    violations: Vec<Violation>,
}

impl Report {
    fn fail(&mut self, check: &'static str, key: Option<&FoundKey>, detail: impl Into<String>) {
        let detail = detail.into();
        let detail = match key {
            Some(key) => format!("{}: {detail}", support::short_name(key)),
            None => detail,
        };
        self.violations.push(Violation { check, detail });
    }
}
