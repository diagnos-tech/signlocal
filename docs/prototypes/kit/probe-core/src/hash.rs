//! Hash algorithms accepted for signing and the digest length rule.

use std::fmt;
use std::str::FromStr;

use sha2::{Digest, Sha256, Sha384, Sha512};

use crate::algorithm::UnknownAlgorithmError;

/// Hash algorithm the caller used to produce the digest.
///
/// Only the SHA-2 family: SHA-1 is not acceptable for new signatures under
/// ICP-Brasil or eIDAS, and SHA-3 has no support in tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlgorithm {
    /// Every algorithm, from the shortest digest to the longest.
    pub const ALL: [HashAlgorithm; 3] = [Self::Sha256, Self::Sha384, Self::Sha512];

    /// Digest size in bytes: 32, 48 or 64.
    pub const fn digest_len(self) -> usize {
        match self {
            Self::Sha256 => 32,
            Self::Sha384 => 48,
            Self::Sha512 => 64,
        }
    }

    /// WebCrypto name: `"SHA-256"`, `"SHA-384"`, `"SHA-512"`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sha256 => "SHA-256",
            Self::Sha384 => "SHA-384",
            Self::Sha512 => "SHA-512",
        }
    }

    /// Name without the hyphen, as some callers spell it (`SHA256`).
    const fn compact_name(self) -> &'static str {
        match self {
            Self::Sha256 => "SHA256",
            Self::Sha384 => "SHA384",
            Self::Sha512 => "SHA512",
        }
    }

    /// Hashes `data`. Used by tests and by the probe to make sample digests.
    pub fn digest(self, data: &[u8]) -> Vec<u8> {
        match self {
            Self::Sha256 => Sha256::digest(data).to_vec(),
            Self::Sha384 => Sha384::digest(data).to_vec(),
            Self::Sha512 => Sha512::digest(data).to_vec(),
        }
    }

    /// Rejects any digest whose length is not exactly [`Self::digest_len`].
    ///
    /// A signer must never sign bytes of the wrong size: that is how a
    /// truncated or forged "digest" would slip through.
    pub fn check_digest(self, digest: &[u8]) -> Result<(), DigestLengthError> {
        let expected = self.digest_len();
        if digest.len() == expected {
            Ok(())
        } else {
            Err(DigestLengthError {
                algorithm: self,
                expected,
                actual: digest.len(),
            })
        }
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for HashAlgorithm {
    type Err = UnknownAlgorithmError;

    /// Accepts `SHA-256`/`SHA256`, `SHA-384`/`SHA384`, `SHA-512`/`SHA512`,
    /// case-insensitively and without trimming.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ASCII-only comparison on purpose: Unicode case folding would let
        // look-alike characters (such as the long s) spell an algorithm.
        Self::ALL
            .into_iter()
            .find(|alg| {
                s.eq_ignore_ascii_case(alg.name()) || s.eq_ignore_ascii_case(alg.compact_name())
            })
            .ok_or_else(|| UnknownAlgorithmError { name: s.to_owned() })
    }
}

/// A digest whose length does not match its declared hash algorithm.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("digest for {algorithm} must be {expected} bytes, got {actual}")]
pub struct DigestLengthError {
    pub algorithm: HashAlgorithm,
    pub expected: usize,
    pub actual: usize,
}
