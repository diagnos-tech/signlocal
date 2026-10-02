//! Checks on `list` (and `capabilities`, which must be as quiet):
//! `lists-expected`, `list-is-quiet`, `provider-is-anonymous`.

use std::collections::BTreeSet;

use websign_core::CertInfo;

use super::support::fingerprint_hex;
use super::{Fixture, Report};
use crate::{FoundKey, Keystore, KeystoreError, PinPrompt};

/// Lists twice and returns the fixture's keys (one per fingerprint), the
/// only ones the other checks may sign with.
pub fn check(keystore: &mut dyn Keystore, fixture: &Fixture, report: &mut Report) -> Vec<FoundKey> {
    let first = list(keystore, report);
    let second = list(keystore, report);
    let set =
        |keys: &[FoundKey]| -> BTreeSet<String> { keys.iter().map(fingerprint_hex).collect() };
    if set(&first) != set(&second) {
        report.fail("list-is-quiet", None, "two listings in a row differ");
    }
    for key in &first {
        check_not_logged_in(keystore, key, report);
    }

    let mut targets = Vec::new();
    for expected in &fixture.expected {
        let expected = expected.to_ascii_lowercase();
        let matching: Vec<&FoundKey> = first
            .iter()
            .filter(|key| fingerprint_hex(key) == expected)
            .collect();
        match matching.as_slice() {
            [key] => targets.push((*key).clone()),
            [] => report.fail("lists-expected", None, format!("{expected} is not listed")),
            _ => report.fail(
                "lists-expected",
                None,
                format!("{expected} is listed {} times", matching.len()),
            ),
        }
    }
    for key in &first {
        check_provider(key, fixture, report);
    }
    targets
}

fn list(keystore: &mut dyn Keystore, report: &mut Report) -> Vec<FoundKey> {
    match keystore.list() {
        Ok(keys) => keys,
        Err(error @ (KeystoreError::WrongPin | KeystoreError::PinRequired)) => {
            report.fail(
                "list-is-quiet",
                None,
                format!("list asked for a PIN: {error}"),
            );
            Vec::new()
        }
        Err(error) => {
            report.fail("lists-expected", None, format!("list failed: {error}"));
            Vec::new()
        }
    }
}

/// Listing and reading capabilities must not have logged in: a token the
/// app unlocks reports locked.
fn check_not_logged_in(keystore: &mut dyn Keystore, key: &FoundKey, report: &mut Report) {
    keystore.capabilities(key);
    if !matches!(key.pin, PinPrompt::App { .. }) {
        return;
    }
    if keystore.pin_state(key).is_some_and(|state| state.unlocked) {
        report.fail(
            "list-is-quiet",
            Some(key),
            "the token is unlocked after list",
        );
    }
}

fn check_provider(key: &FoundKey, fixture: &Fixture, report: &mut Report) {
    let provider = key.provider.to_lowercase();
    let mut forbidden: Vec<String> = fixture.private_text.clone();
    if let Ok(info) = CertInfo::from_der(&key.cert_der) {
        forbidden.push(info.display_name());
        forbidden.extend(info.subject.common_name.clone());
        forbidden.extend(info.subject.serial_number.clone());
        // Short serials ("01") would match unrelated digits.
        if info.serial_hex.len() >= 8 {
            forbidden.push(info.serial_hex.clone());
        }
    }
    let leaked = forbidden
        .iter()
        .map(|text| text.trim().to_lowercase())
        .any(|text| !text.is_empty() && provider.contains(&text));
    if leaked {
        report.fail(
            "provider-is-anonymous",
            Some(key),
            "provider text contains private data",
        );
    }
}
