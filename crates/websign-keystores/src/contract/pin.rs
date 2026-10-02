//! Checks on keys whose PIN the app collects: `wrong-pin`, `session-reuse`
//! (`docs/plan.md` D5) and `always-authenticate`.

use super::support::{any_algorithm, fixture_pin, sign};
use super::{Fixture, Report};
use crate::{FoundKey, Keystore, KeystoreError, PinPrompt, PinState};
use websign_core::HashAlgorithm;

/// Keys the suite can drive: the app collects the PIN and it is typed, not
/// entered on a PIN pad.
pub fn app_collects_pin(key: &FoundKey) -> bool {
    key.pin
        == PinPrompt::App {
            protected_path: false,
        }
}

/// A wrong PIN is `WrongPin` and does not stop the right one from working.
pub fn check_wrong_pin(
    keystore: &mut dyn Keystore,
    key: &FoundKey,
    fixture: &Fixture,
    report: &mut Report,
) {
    const CHECK: &str = "wrong-pin";
    let (Some(wrong), Some(right)) = (fixture.wrong_pin.as_deref(), fixture_pin(key, fixture))
    else {
        return;
    };
    keystore.end_sessions();
    match attempt(keystore, key, Some(wrong)) {
        Err(KeystoreError::WrongPin) => {}
        Ok(()) => return report.fail(CHECK, Some(key), "signed with a wrong PIN"),
        Err(error) => report.fail(CHECK, Some(key), format!("expected WrongPin, got {error}")),
    }
    // Whether the token now warns (`count_low`) is the token's business;
    // it must still say something.
    if keystore.pin_state(key).is_none() {
        report.fail(CHECK, Some(key), "no PIN state after a wrong PIN");
    }
    if let Err(error) = attempt(keystore, key, Some(&right)) {
        report.fail(
            CHECK,
            Some(key),
            format!("the right PIN failed after a wrong one: {error}"),
        );
    }
    keystore.end_sessions();
}

/// After one signature with the PIN, the token signs without it until
/// `end_sessions`; keys with `CKA_ALWAYS_AUTHENTICATE` never do.
pub fn check_sessions(
    keystore: &mut dyn Keystore,
    key: &FoundKey,
    fixture: &Fixture,
    report: &mut Report,
) {
    let Some(pin) = fixture_pin(key, fixture) else {
        return;
    };
    keystore.end_sessions();
    if let Err(error) = attempt(keystore, key, Some(&pin)) {
        return report.fail(
            "session-reuse",
            Some(key),
            format!("first signature: {error}"),
        );
    }
    let Some(state) = keystore.pin_state(key) else {
        return report.fail(
            "session-reuse",
            Some(key),
            "no PIN state for an app-PIN key",
        );
    };
    if state.always_authenticate {
        always_authenticate(keystore, key, &pin, report);
    } else {
        reuse(keystore, key, state, report);
    }
    keystore.end_sessions();
}

fn reuse(keystore: &mut dyn Keystore, key: &FoundKey, state: PinState, report: &mut Report) {
    const CHECK: &str = "session-reuse";
    let without_pin = attempt(keystore, key, None);
    if !state.unlocked {
        // A token that needs no login signs without a PIN anyway.
        if let Err(error) = without_pin {
            report.fail(
                CHECK,
                Some(key),
                format!("not kept unlocked after a login: {error}"),
            );
        }
        return;
    }
    if let Err(error) = without_pin {
        report.fail(
            CHECK,
            Some(key),
            format!("unlocked, yet a PIN-less signature failed: {error}"),
        );
    }
    keystore.end_sessions();
    if keystore.pin_state(key).is_some_and(|state| state.unlocked) {
        report.fail(CHECK, Some(key), "still unlocked after end_sessions");
    }
    match attempt(keystore, key, None) {
        Err(KeystoreError::PinRequired) => {}
        Ok(()) => report.fail(CHECK, Some(key), "signed without a PIN after end_sessions"),
        Err(error) => report.fail(
            CHECK,
            Some(key),
            format!("expected PinRequired, got {error}"),
        ),
    }
}

fn always_authenticate(
    keystore: &mut dyn Keystore,
    key: &FoundKey,
    pin: &str,
    report: &mut Report,
) {
    const CHECK: &str = "always-authenticate";
    match attempt(keystore, key, None) {
        Err(KeystoreError::PinRequired) => {}
        Ok(()) => report.fail(CHECK, Some(key), "signed without the PIN"),
        Err(error) => report.fail(
            CHECK,
            Some(key),
            format!("expected PinRequired, got {error}"),
        ),
    }
    if let Err(error) = attempt(keystore, key, Some(pin)) {
        report.fail(
            CHECK,
            Some(key),
            format!("second signature with the PIN: {error}"),
        );
    }
}

/// One SHA-256 signature with any algorithm the key supports.
fn attempt(
    keystore: &mut dyn Keystore,
    key: &FoundKey,
    pin: Option<&str>,
) -> Result<(), KeystoreError> {
    let algorithm = any_algorithm(key).ok_or(KeystoreError::NotFound)?;
    sign(keystore, key, HashAlgorithm::Sha256, algorithm, pin)
        .1
        .map(drop)
}
