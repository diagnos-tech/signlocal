//! Brainpool fixtures: OpenSSL signatures and DER/raw pairs
//! (`vectors/brainpool-*.txt`), kept apart from the NIST manifests so the
//! promoted tests that walk those keep their curve lists.

use websign_core::Curve;

use super::vectors::{EcdsaVector, SignatureVector, records};
use super::{fixture_curve, fixture_hash, hex_decode};

/// Certificate fixture name and curve of every Brainpool `r1` key.
pub const BRAINPOOL_R1_KEYS: [(&str, Curve); 3] = [
    ("brainpoolP256r1", Curve::BrainpoolP256r1),
    ("brainpoolP384r1", Curve::BrainpoolP384r1),
    ("brainpoolP512r1", Curve::BrainpoolP512r1),
];

/// Twisted-curve certificates and their curve OIDs, which must stay unknown.
pub const BRAINPOOL_TWISTED: [(&str, &str); 3] = [
    ("brainpoolP256t1", "1.3.36.3.3.2.8.1.1.8"),
    ("brainpoolP384t1", "1.3.36.3.3.2.8.1.1.12"),
    ("brainpoolP512t1", "1.3.36.3.3.2.8.1.1.14"),
];

pub fn brainpool_signature_vectors() -> Vec<SignatureVector> {
    records("brainpool-signatures.txt")
        .into_iter()
        .map(|r| SignatureVector {
            key: r[0].clone(),
            algorithm: r[1].clone(),
            hash: fixture_hash(&r[2]),
            signature: hex_decode(&r[3]),
        })
        .collect()
}

pub fn brainpool_ecdsa_vectors() -> Vec<EcdsaVector> {
    records("brainpool-ecdsa.txt")
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
