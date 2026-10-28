//! Algorithm names, spelled as WebCrypto spells them.

use serde::{Deserialize, Serialize};

/// A hash the digest was computed with. SHA-2 only: SHA-1 is broken and
/// SHA-3 is not offered by the OS key stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum HashName {
    #[serde(rename = "SHA-256")]
    Sha256,
    #[serde(rename = "SHA-384")]
    Sha384,
    #[serde(rename = "SHA-512")]
    Sha512,
}

impl HashName {
    /// Exact digest length in bytes: 32, 48, 64.
    pub const fn digest_len(self) -> usize {
        match self {
            HashName::Sha256 => 32,
            HashName::Sha384 => 48,
            HashName::Sha512 => 64,
        }
    }
}

/// A signature scheme. RSASSA-PSS always uses MGF1 with the same hash and a
/// salt as long as the digest; ECDSA signatures travel as raw `r || s`.
///
/// EdDSA is absent on purpose: pure EdDSA signs the message, not a hash, so
/// it cannot work in a hash-only signer (`docs/architecture/compatibility.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum SignatureAlgorithmName {
    #[serde(rename = "ECDSA")]
    Ecdsa,
    #[serde(rename = "RSASSA-PKCS1-v1_5")]
    RsaPkcs1v15,
    #[serde(rename = "RSASSA-PSS")]
    RsaPss,
}

impl SignatureAlgorithmName {
    /// Preference order when the caller does not state one: ECDSA for EC
    /// keys, PKCS#1 v1.5 for RSA keys (what every PAdES validator accepts).
    pub const DEFAULT_PREFERENCE: [SignatureAlgorithmName; 3] = [
        SignatureAlgorithmName::Ecdsa,
        SignatureAlgorithmName::RsaPkcs1v15,
        SignatureAlgorithmName::RsaPss,
    ];
}

/// An elliptic curve, by its common name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum CurveName {
    #[serde(rename = "P-256")]
    P256,
    #[serde(rename = "P-384")]
    P384,
    #[serde(rename = "P-521")]
    P521,
    #[serde(rename = "brainpoolP256r1")]
    BrainpoolP256r1,
    #[serde(rename = "brainpoolP384r1")]
    BrainpoolP384r1,
    #[serde(rename = "brainpoolP512r1")]
    BrainpoolP512r1,
}
