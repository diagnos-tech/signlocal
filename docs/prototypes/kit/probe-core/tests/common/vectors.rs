//! Parsers for the OpenSSL-made manifests in `tests/fixtures/vectors`.
//! Every manifest is text: `#` comments, then one whitespace-separated
//! record per line.

use std::fs;

use probe_core::{Curve, Fingerprint, HashAlgorithm};

use super::{fixture_curve, fixture_hash, fixture_path, hex_decode};

fn records(file: &str) -> Vec<Vec<String>> {
    let path = fixture_path(&format!("vectors/{file}"));
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read manifest {}: {e}", path.display()));
    text.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| line.split_whitespace().map(str::to_owned).collect())
        .collect()
}

/// The digest of the fixture message under `hash`, computed by OpenSSL.
pub fn fixture_digest(hash: HashAlgorithm) -> Vec<u8> {
    let wanted = match hash {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Sha384 => "sha384",
        HashAlgorithm::Sha512 => "sha512",
    };
    records("digests.txt")
        .into_iter()
        .find(|r| r[0] == wanted)
        .map(|r| hex_decode(&r[1]))
        .expect("digest in manifest")
}

/// The message all fixture digests and signatures were made over.
pub const FIXTURE_MESSAGE: &[u8] = b"WebeSign probe-core fixture message";

/// PKCS#1 `DigestInfo` of the fixture digest under `hash`, built by OpenSSL.
pub fn fixture_digest_info(hash: HashAlgorithm) -> Vec<u8> {
    let wanted = match hash {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Sha384 => "sha384",
        HashAlgorithm::Sha512 => "sha512",
    };
    records("digestinfo.txt")
        .into_iter()
        .find(|r| r[0] == wanted)
        .map(|r| hex_decode(&r[1]))
        .expect("digest info in manifest")
}

/// SHA-256 of a certificate's DER as OpenSSL reports it.
pub fn fixture_fingerprint(name: &str) -> Fingerprint {
    let hex = records("fingerprints.txt")
        .into_iter()
        .find(|r| r[0] == name)
        .map(|r| r[1].clone())
        .unwrap_or_else(|| panic!("no fingerprint for {name}"));
    let bytes: [u8; 32] = hex_decode(&hex).try_into().expect("32 bytes");
    Fingerprint::from_bytes(bytes)
}

/// Serial, `notBefore` and `notAfter` of a certificate as OpenSSL reports them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidityOracle {
    pub name: String,
    pub serial_hex: String,
    pub not_before: i64,
    pub not_after: i64,
}

pub fn validity_oracles() -> Vec<ValidityOracle> {
    records("validity.txt")
        .into_iter()
        .map(|r| ValidityOracle {
            name: r[0].clone(),
            serial_hex: r[1].clone(),
            not_before: r[2].parse().expect("unix time"),
            not_after: r[3].parse().expect("unix time"),
        })
        .collect()
}

/// One OpenSSL signature over the fixture digest.
#[derive(Debug, Clone)]
pub struct SignatureVector {
    /// Name of the certificate fixture holding the public key.
    pub key: String,
    /// `pkcs1`, `pss`, `ecdsa`, or a deliberately wrong PSS variant.
    pub algorithm: String,
    pub hash: HashAlgorithm,
    pub signature: Vec<u8>,
}

pub fn signature_vectors() -> Vec<SignatureVector> {
    records("signatures.txt")
        .into_iter()
        .map(|r| SignatureVector {
            key: r[0].clone(),
            algorithm: r[1].clone(),
            hash: fixture_hash(&r[2]),
            signature: hex_decode(&r[3]),
        })
        .collect()
}

/// The signature for `key`, `algorithm` and `hash`; panics if the manifest lacks it.
pub fn signature(key: &str, algorithm: &str, hash: HashAlgorithm) -> Vec<u8> {
    signature_vectors()
        .into_iter()
        .find(|v| v.key == key && v.algorithm == algorithm && v.hash == hash)
        .unwrap_or_else(|| panic!("no {algorithm} signature for {key} with {hash}"))
        .signature
}

/// One ECDSA signature in both encodings, with the shape it exercises.
#[derive(Debug, Clone)]
pub struct EcdsaVector {
    pub curve: Curve,
    /// `plain`, `high-r`, `high-s`, `high-both`, `short-r` or `short-s`.
    pub shape: String,
    pub hash: HashAlgorithm,
    pub raw: Vec<u8>,
    pub der: Vec<u8>,
}

pub fn ecdsa_vectors() -> Vec<EcdsaVector> {
    records("ecdsa.txt")
        .into_iter()
        .map(|r| EcdsaVector {
            curve: fixture_curve(&r[0]),
            shape: r[1].clone(),
            hash: fixture_hash(&r[2]),
            raw: hex_decode(&r[3]),
            der: hex_decode(&r[4]),
        })
        .collect()
}
