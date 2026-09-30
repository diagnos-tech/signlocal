//! SPEC §4.1 and §7: `verify` on Brainpool keys. The signatures were made by
//! OpenSSL (`vectors/brainpool-*.txt`); the crate under test never signs.

mod check_order;
mod invalid;
mod valid;

#[path = "../common/mod.rs"]
mod common;

use common::{
    BRAINPOOL_R1_KEYS, BRAINPOOL_TWISTED, brainpool_ecdsa_vectors, brainpool_signature_vectors,
    cert, fixture_digest, signature,
};
use websign_core::ecdsa::der_to_raw;
use websign_core::{
    CertInfo, Curve, DigestLengthError, HashAlgorithm, PublicKeyKind, SignatureAlgorithm,
    VerifyError, verify,
};

use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

const VERIFIABLE: [&str; 2] = ["brainpoolP256r1", "brainpoolP384r1"];

fn verify_fixture(
    key: &str,
    hash: HashAlgorithm,
    algorithm: SignatureAlgorithm,
    signature: &[u8],
) -> Result<(), VerifyError> {
    verify(
        &cert(key),
        hash,
        algorithm,
        &fixture_digest(hash),
        signature,
    )
}

fn key_for(curve: Curve) -> &'static str {
    BRAINPOOL_R1_KEYS
        .iter()
        .find(|(_, c)| *c == curve)
        .map(|(name, _)| *name)
        .expect("Brainpool fixture key")
}

/// Copy of `cert` whose EC point has its last byte changed, so the point is
/// off the curve (with overwhelming probability, and certainly not the same).
fn with_corrupted_point(cert: &[u8], point_len: usize) -> Vec<u8> {
    // BIT STRING header + unused-bits byte + uncompressed-point marker.
    let marker = [0x03, (point_len + 1) as u8, 0x00, 0x04];
    let at = cert
        .windows(marker.len())
        .position(|w| w == marker)
        .expect("EC point in the fixture");
    let last = at + 3 + point_len - 1;
    let mut out = cert.to_vec();
    out[last] ^= 0x01;
    out
}

/// The OpenSSL signature of `key` over the fixture digest made with `hash`.
fn signature_of(key: &str, hash: HashAlgorithm) -> Vec<u8> {
    brainpool_signature_vectors()
        .into_iter()
        .find(|v| v.key == key && v.hash == hash)
        .unwrap_or_else(|| panic!("no signature for {key} {hash}"))
        .signature
}
