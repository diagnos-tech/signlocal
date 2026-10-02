//! A throwaway SoftHSM2 token (made by `softhsm-fixture.sh`) and the
//! re-execution trick that lets tests use it.
//!
//! SoftHSM reads `SOFTHSM2_CONF` once, in `C_Initialize`, and a module is
//! initialized once per process. Setting the variable from a running test
//! would race the other test threads, so the parent test creates the token
//! and runs this test binary again with the variable set, filtered to the
//! tests that need the token.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use websign_keystores::{Keystore, Opened, Options};

/// Where the child process finds the fixture directory.
const FIXTURE_VAR: &str = "WEBSIGN_SOFTHSM_FIXTURE";
/// Set in CI so a machine without SoftHSM fails instead of skipping.
const REQUIRE_VAR: &str = "WEBSIGN_REQUIRE_SOFTHSM";

/// The token as the fixture script left it.
#[derive(Debug)]
pub struct SoftHsm {
    pub dir: PathBuf,
    pub module: PathBuf,
    pub pin: String,
    pub so_pin: String,
    pub label: String,
    pub serial: String,
}

impl SoftHsm {
    /// The fixture of this child process; `None` when not run by
    /// [`run_children`] (the test then does nothing).
    pub fn in_child() -> Option<SoftHsm> {
        let dir = PathBuf::from(std::env::var_os(FIXTURE_VAR)?);
        let conf = std::fs::read_to_string(dir.join("fixture.conf")).ok()?;
        let values: HashMap<&str, &str> = conf
            .lines()
            .filter_map(|line| line.split_once('='))
            .collect();
        let value = |key: &str| values.get(key).map(|value| (*value).to_owned());
        Some(SoftHsm {
            module: PathBuf::from(value("SOFTHSM_MODULE")?),
            pin: value("SOFTHSM_USER_PIN")?,
            so_pin: value("SOFTHSM_SO_PIN")?,
            label: value("SOFTHSM_TOKEN_LABEL")?,
            serial: value("SOFTHSM_TOKEN_SERIAL")?,
            dir,
        })
    }

    pub fn options(&self) -> Options {
        Options {
            extra_modules: vec![self.module.clone()],
            no_known_modules: true,
            no_p11_kit: true,
            ..Options::default()
        }
    }

    /// The SoftHSM module, opened alone.
    pub fn keystore(&self) -> Box<dyn Keystore> {
        let mut opened = Opened::default();
        websign_keystores::pkcs11::open(&self.options(), &mut opened);
        assert!(opened.failures.is_empty(), "{:?}", opened.failures);
        assert_eq!(opened.keystores.len(), 1);
        opened.keystores.remove(0)
    }

    /// `(name, certificate DER)` of every key the token holds.
    pub fn keys(&self) -> Vec<(String, Vec<u8>)> {
        let mut keys: Vec<(String, Vec<u8>)> = std::fs::read_dir(self.dir.join("keys"))
            .expect("keys directory")
            .map(|entry| {
                let entry = entry.expect("directory entry");
                let name = entry.file_name().to_string_lossy().into_owned();
                let der = std::fs::read(entry.path().join("cert.der")).expect("cert.der");
                (name, der)
            })
            .collect();
        keys.sort();
        keys
    }

    pub fn key(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.dir.join("keys").join(name).join("cert.der")).expect("cert.der")
    }

    pub fn ca(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.dir.join("ca").join(format!("{name}.der"))).expect("CA DER")
    }
}

/// Creates the token and runs the ignored tests whose names contain
/// `filter` in a child process that sees it. Skips (returns) when SoftHSM
/// is not installed, or on Windows, unless `WEBSIGN_REQUIRE_SOFTHSM` is set.
pub fn run_children(filter: &str) {
    let required = std::env::var_os(REQUIRE_VAR).is_some();
    // The fixture is a POSIX script, and on Windows `bash` is often the WSL
    // launcher, which fails without a distribution instead of reporting
    // missing tooling.
    if cfg!(windows) && !required {
        eprintln!(
            "SoftHSM2 fixture needs a POSIX shell: skipped (set {REQUIRE_VAR} to fail instead)"
        );
        return;
    }
    let dir = tempfile::tempdir().expect("temporary directory");
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/softhsm-fixture.sh");
    let status = Command::new("bash")
        .arg(&script)
        .arg(dir.path())
        .status()
        .expect("bash runs");
    if status.code() == Some(2) && !required {
        eprintln!("SoftHSM2 tooling not installed: skipped (set {REQUIRE_VAR} to fail instead)");
        return;
    }
    assert!(status.success(), "softhsm-fixture.sh failed: {status}");

    let status = Command::new(std::env::current_exe().expect("test binary"))
        .args([filter, "--ignored", "--test-threads=1", "--nocapture"])
        .env("SOFTHSM2_CONF", dir.path().join("softhsm2.conf"))
        .env(FIXTURE_VAR, dir.path())
        .status()
        .expect("test binary runs");
    assert!(status.success(), "tests against SoftHSM2 failed");
}
