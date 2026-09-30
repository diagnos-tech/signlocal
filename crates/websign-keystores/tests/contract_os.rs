//! The keystore contract against the OS key store of the machine running the
//! tests, with keys a CI job created (a software KSP/CSP on Windows, a
//! temporary keychain on macOS).
//!
//! Configured from the environment, so the per-OS jobs decide what to check:
//!
//! - `WEBSIGN_CONTRACT_SOURCE`: [`Keystore::name`] of the source, e.g.
//!   `windows` or `macos:keychain`; without it the test does nothing.
//! - `WEBSIGN_CONTRACT_EXPECTED`: comma-separated SHA-256 fingerprints.
//! - `WEBSIGN_CONTRACT_PRIVATE_TEXT`: comma-separated text that must never
//!   appear in `provider` (container names, token labels).

use websign_keystores::contract::{self, Fixture};
use websign_keystores::{Keystore, Options, open_all};

fn list(variable: &str) -> Vec<String> {
    std::env::var(variable)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_os_key_store_meets_the_contract() {
    let Ok(source) = std::env::var("WEBSIGN_CONTRACT_SOURCE") else {
        eprintln!("WEBSIGN_CONTRACT_SOURCE is not set: skipped");
        return;
    };
    let options = Options {
        no_known_modules: true,
        no_p11_kit: true,
        // A key that wants UI fails instead of waiting for a click.
        silent: true,
        ..Options::default()
    };
    let mut opened = open_all(&options);
    let names: Vec<String> = opened
        .keystores
        .iter()
        .map(|keystore| keystore.name())
        .collect();
    let keystore: &mut Box<dyn Keystore> = opened
        .keystores
        .iter_mut()
        .find(|keystore| keystore.name() == source)
        .unwrap_or_else(|| panic!("no source {source:?}; opened: {names:?}"));
    let fixture = Fixture {
        expected: list("WEBSIGN_CONTRACT_EXPECTED"),
        private_text: list("WEBSIGN_CONTRACT_PRIVATE_TEXT"),
        ..Fixture::default()
    };
    assert!(
        !fixture.expected.is_empty(),
        "WEBSIGN_CONTRACT_EXPECTED is empty"
    );
    let violations = contract::run(keystore.as_mut(), &fixture);
    assert!(violations.is_empty(), "{violations:#?}");
}
