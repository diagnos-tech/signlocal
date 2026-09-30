//! Runs the built CLI against the fake app.

use std::process::Command;

use sha2::{Digest, Sha256};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_websign-rust-cli-example"))
}

#[test]
fn signs_the_sha256_of_the_file_against_the_fake_app() {
    let output = cli().arg("Cargo.toml").output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut lines = stdout.lines();
    assert_eq!(lines.next(), Some("signed by Test Holder with Ecdsa"));
    // The fake echoes the digest, so the "signature" is the file's SHA-256.
    let digest = Sha256::digest(std::fs::read("Cargo.toml").unwrap());
    let expected: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(lines.next(), Some(expected.as_str()));
}

#[test]
fn a_missing_argument_is_a_usage_error() {
    assert_eq!(cli().output().unwrap().status.code(), Some(2));
}
