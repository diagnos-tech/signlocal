//! The runtime serving a whole signature with the real key worker against
//! a SoftHSM2 token.
//!
//! SoftHSM reads `SOFTHSM2_CONF` once, in `C_Initialize`, and a module is
//! initialized once per process. So the parent test builds the token with the
//! key stores' fixture script and runs this test binary again with the
//! variable set, filtered to [`child`].

mod child;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where the child finds the fixture directory.
const FIXTURE_VAR: &str = "WEBSIGN_SOFTHSM_FIXTURE";
/// Set in CI so a machine without SoftHSM fails instead of skipping.
const REQUIRE_VAR: &str = "WEBSIGN_REQUIRE_SOFTHSM";

/// The token as the fixture script left it.
pub(super) struct Token {
    pub dir: PathBuf,
    pub module: PathBuf,
    pub pin: String,
}

impl Token {
    /// The fixture of this child process; `None` outside [`serves_a_signature_with_softhsm2`].
    pub fn in_child() -> Option<Token> {
        let dir = PathBuf::from(std::env::var_os(FIXTURE_VAR)?);
        let conf = std::fs::read_to_string(dir.join("fixture.conf")).ok()?;
        let values: HashMap<&str, &str> = conf
            .lines()
            .filter_map(|line| line.split_once('='))
            .collect();
        Some(Token {
            module: PathBuf::from(values.get("SOFTHSM_MODULE")?),
            pin: (*values.get("SOFTHSM_USER_PIN")?).to_owned(),
            dir,
        })
    }

    pub fn certificate(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.dir.join("keys").join(name).join("cert.der")).unwrap()
    }
}

#[test]
fn serves_a_signature_with_softhsm2() {
    let dir = tempfile::tempdir().unwrap();
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../websign-keystores/tests/support/softhsm-fixture.sh");
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

    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "runtime::softhsm_tests::child::",
            "--ignored",
            "--test-threads=1",
            "--nocapture",
        ])
        .env("SOFTHSM2_CONF", dir.path().join("softhsm2.conf"))
        .env(FIXTURE_VAR, dir.path())
        .status()
        .unwrap();
    assert!(status.success(), "the child run against SoftHSM2 failed");
}
