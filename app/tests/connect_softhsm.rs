//! `websign connect` end to end: the real client library starts the built
//! binary, which lists a SoftHSM2 token and signs with it after the
//! headless confirmation of the `e2e` build.
//!
//! SoftHSM reads `SOFTHSM2_CONF` in the app's process, which inherits this
//! process's environment, so the parent test builds the token and runs this
//! test binary again with the variables set, filtered to [`child`] (setting
//! variables in a running multi-threaded test is not sound).
//!
//! Skipped without `softhsm2-util`, `pkcs11-tool` and `openssl`, unless
//! `WEBSIGN_REQUIRE_SOFTHSM` is set.

#![cfg(all(feature = "e2e", target_os = "linux"))]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use websign_client::{Client, ConnectOptions, HashName, SignOptions};
use websign_protocol::types::FingerprintHex;

const FIXTURE_VAR: &str = "WEBSIGN_TEST_SOFTHSM_FIXTURE";
const REQUIRE_VAR: &str = "WEBSIGN_REQUIRE_SOFTHSM";

#[test]
fn signs_through_websign_connect_with_softhsm2() {
    let dir = tempfile::tempdir().unwrap();
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../crates/websign-keystores/tests/support/softhsm-fixture.sh");
    let status = Command::new("bash")
        .arg(&script)
        .arg(dir.path())
        .status()
        .unwrap();
    if status.code() == Some(2) && std::env::var_os(REQUIRE_VAR).is_none() {
        eprintln!("SoftHSM2 tooling not installed: skipped (set {REQUIRE_VAR} to fail instead)");
        return;
    }
    assert!(status.success(), "softhsm-fixture.sh failed: {status}");
    let fixture = read_fixture(dir.path());

    // The app's data folder names the token's module as a user driver.
    let config = dir.path().join("config");
    std::fs::create_dir_all(config.join("websign")).unwrap();
    let settings = serde_json::json!({ "version": 1, "userModules": [fixture["SOFTHSM_MODULE"]] });
    std::fs::write(config.join("websign/settings.json"), settings.to_string()).unwrap();

    let status = Command::new(std::env::current_exe().unwrap())
        .args(["child", "--ignored", "--exact", "--nocapture"])
        .env("SOFTHSM2_CONF", dir.path().join("softhsm2.conf"))
        .env("XDG_CONFIG_HOME", &config)
        .env("XDG_STATE_HOME", dir.path().join("state"))
        .env("WEBSIGN_E2E_HEADLESS", "1")
        .env("WEBSIGN_E2E_PIN", &fixture["SOFTHSM_USER_PIN"])
        .env(FIXTURE_VAR, dir.path())
        .status()
        .unwrap();
    assert!(status.success(), "the client run against SoftHSM2 failed");
}

#[test]
#[ignore = "run by signs_through_websign_connect_with_softhsm2 with the token's environment"]
fn child() {
    let Some(dir) = std::env::var_os(FIXTURE_VAR).map(PathBuf::from) else {
        return;
    };
    let der = std::fs::read(dir.join("keys/rsa-2048/cert.der")).unwrap();
    let fingerprint = websign_core::Fingerprint::of(&der)
        .to_hex()
        .to_ascii_lowercase();
    let mut client = Client::connect_with(ConnectOptions {
        executable: Some(PathBuf::from(env!("CARGO_BIN_EXE_websign"))),
        ..ConnectOptions::default()
    })
    .unwrap();
    assert_eq!(client.status().unwrap().app.protocols.max, 1);

    let mut options = SignOptions::new(HashName::Sha256);
    options.certificate = Some(FingerprintHex::new(fingerprint.clone()).unwrap());
    let result = client.sign(options, |_, _| Ok(vec![0x5a; 32])).unwrap();
    // The app verified the signature against the certificate before sending.
    assert_eq!(result.certificate.fingerprint.as_str(), fingerprint);
    assert_eq!(result.signature.as_bytes().len(), 256);
}

fn read_fixture(dir: &Path) -> HashMap<String, String> {
    std::fs::read_to_string(dir.join("fixture.conf"))
        .unwrap()
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}
