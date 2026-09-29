//! Helpers shared by the integration tests: fixture loading, hex, hand-made
//! DER signatures and a blank `CertInfo`.
//!
//! Fixtures are read at run time from `tests/fixtures`; nothing here needs
//! OpenSSL. `tests/fixtures/README.md` says what each file is.
#![allow(dead_code)]

mod der;
mod vectors;

#[allow(unused_imports)]
pub use der::*;
#[allow(unused_imports)]
pub use vectors::*;

use std::fs;
use std::path::PathBuf;

use probe_core::{CertInfo, Curve, DistinguishedName, Fingerprint, HashAlgorithm, PublicKeyKind};

/// Fixtures whose outcome the SPEC leaves open (an arc beyond `u32` cannot be
/// held by `IcpLevel::Other`); generic loops skip them.
pub const UNSPECIFIED_FIXTURES: [&str; 1] = ["icp-level-huge"];

pub fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative)
}

/// DER bytes of `tests/fixtures/certs/<name>.der`.
pub fn cert(name: &str) -> Vec<u8> {
    let path = fixture_path(&format!("certs/{name}.der"));
    fs::read(&path).unwrap_or_else(|e| panic!("cannot read fixture {}: {e}", path.display()))
}

/// Every certificate fixture name, sorted.
pub fn cert_names() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(fixture_path("certs"))
        .expect("certs fixture directory")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            name.strip_suffix(".der").map(str::to_owned)
        })
        .collect();
    names.sort();
    names
}

/// Parses a certificate fixture, failing the test with the fixture name.
pub fn info(name: &str) -> CertInfo {
    CertInfo::from_der(&cert(name)).unwrap_or_else(|e| panic!("fixture {name} must parse: {e}"))
}

/// Decodes hex, ignoring whitespace so long vectors can be split for reading.
pub fn hex_decode(text: &str) -> Vec<u8> {
    let text: String = text.split_whitespace().collect();
    let text = text.as_str();
    assert!(text.len().is_multiple_of(2), "odd hex length: {text}");
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex digit"))
        .collect()
}

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `count` bytes counting up from `start` (wrapping), never all zero for
/// `start != 0`: a recognizable stand-in for a big integer.
pub fn counting_bytes(start: u8, count: usize) -> Vec<u8> {
    (0..count).map(|i| start.wrapping_add(i as u8)).collect()
}

/// `bytes` left-padded with zeros to `len`.
pub fn left_pad(bytes: &[u8], len: usize) -> Vec<u8> {
    assert!(bytes.len() <= len);
    let mut out = vec![0; len - bytes.len()];
    out.extend_from_slice(bytes);
    out
}

pub fn fixture_hash(name: &str) -> HashAlgorithm {
    match name {
        "sha256" => HashAlgorithm::Sha256,
        "sha384" => HashAlgorithm::Sha384,
        "sha512" => HashAlgorithm::Sha512,
        other => panic!("unknown hash in fixture manifest: {other}"),
    }
}

pub fn fixture_curve(name: &str) -> Curve {
    match name {
        "p256" => Curve::P256,
        "p384" => Curve::P384,
        "p521" => Curve::P521,
        other => panic!("unknown curve in fixture manifest: {other}"),
    }
}

/// A `CertInfo` with every field empty or neutral, to vary one field at a time.
pub fn blank_info() -> CertInfo {
    CertInfo {
        fingerprint: Fingerprint::from_bytes([0xAB; 32]),
        subject: DistinguishedName::default(),
        issuer: DistinguishedName::default(),
        serial_hex: "01".to_owned(),
        not_before: 1_000,
        not_after: 2_000,
        key: PublicKeyKind::Ec { curve: Curve::P256 },
        key_usage: None,
        extended_key_usage: Vec::new(),
        policies: Vec::new(),
        is_ca: false,
        icp_brasil: None,
        qualified: None,
    }
}

/// Small deterministic generator (xorshift64*) so property-style tests need
/// no dependency and fail the same way every run.
#[derive(Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn bytes(&mut self, count: usize) -> Vec<u8> {
        (0..count).map(|_| (self.next_u64() >> 32) as u8).collect()
    }

    pub fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}
