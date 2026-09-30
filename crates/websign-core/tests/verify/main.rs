//! SPEC §7: `verify`, `VerifyError`.
//!
//! Every valid signature here was made by OpenSSL over the digest of the
//! fixture message (`tests/fixtures/vectors/signatures.txt`); the crate under
//! test never signs.

mod error_order;
mod invalid_1;
mod invalid_2;
mod invalid_3;
mod valid;
mod verify_error;

#[path = "../common/mod.rs"]
mod common;

use common::{
    SignatureVector, cert, ecdsa_vectors, fixture_digest, hex_decode, left_pad, rsa_modulus,
    signature, signature_vectors,
};
use websign_core::ecdsa::der_to_raw;
use websign_core::{
    CertError, CertInfo, Curve, DigestLengthError, HashAlgorithm, SignatureAlgorithm, VerifyError,
    verify,
};

use HashAlgorithm::{Sha256, Sha384, Sha512};
use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

fn algorithm_of(manifest_name: &str) -> SignatureAlgorithm {
    match manifest_name {
        "pkcs1" => RsaPkcs1v15,
        "pss" => RsaPss,
        "ecdsa" => Ecdsa,
        other => panic!("not a valid-signature kind: {other}"),
    }
}

/// The signatures the SDK promises to be valid.
fn valid_vectors() -> Vec<SignatureVector> {
    signature_vectors()
        .into_iter()
        .filter(|v| matches!(v.algorithm.as_str(), "pkcs1" | "pss" | "ecdsa"))
        .collect()
}

fn check(vector: &SignatureVector) -> Result<(), VerifyError> {
    verify(
        &cert(&vector.key),
        vector.hash,
        algorithm_of(&vector.algorithm),
        &fixture_digest(vector.hash),
        &vector.signature,
    )
}

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

const UNSUPPORTED_KEYS: [&str; 5] = ["ed25519", "ed448", "secp256k1", "secp224r1", "dsa1024"];
