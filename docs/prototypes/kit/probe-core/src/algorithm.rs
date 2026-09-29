//! Signature algorithm names, exactly as the SDK and WebCrypto spell them.

use std::fmt;
use std::str::FromStr;

/// How a digest is turned into a signature.
///
/// The set is closed on purpose: it is what PDF/CMS signers (PAdES, CAdES)
/// accept and what every supported key store can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignatureAlgorithm {
    /// ECDSA; signatures travel as raw `r || s` (IEEE P1363).
    Ecdsa,
    /// RSASSA-PKCS1-v1_5.
    RsaPkcs1v15,
    /// RSASSA-PSS with MGF1 over the same hash and salt length = digest length.
    RsaPss,
}

impl SignatureAlgorithm {
    /// Every algorithm, in declaration order.
    pub const ALL: [SignatureAlgorithm; 3] = [Self::Ecdsa, Self::RsaPkcs1v15, Self::RsaPss];

    /// WebCrypto name: `"ECDSA"`, `"RSASSA-PKCS1-v1_5"`, `"RSASSA-PSS"`.
    pub const fn name(self) -> &'static str {
        todo!()
    }

    /// Whether the algorithm needs an RSA key.
    pub const fn is_rsa(self) -> bool {
        todo!()
    }
}

impl fmt::Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl FromStr for SignatureAlgorithm {
    type Err = UnknownAlgorithmError;

    /// Case-insensitive match on the WebCrypto name.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

/// A hash or signature algorithm name that is not supported.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown algorithm: {name}")]
pub struct UnknownAlgorithmError {
    /// The name exactly as received.
    pub name: String,
}
