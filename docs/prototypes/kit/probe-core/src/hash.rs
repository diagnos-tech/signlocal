//! Hash algorithms accepted for signing and the digest length rule.

use std::fmt;
use std::str::FromStr;

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
        todo!()
    }

    /// WebCrypto name: `"SHA-256"`, `"SHA-384"`, `"SHA-512"`.
    pub const fn name(self) -> &'static str {
        todo!()
    }

    /// Hashes `data`. Used by tests and by the probe to make sample digests.
    pub fn digest(self, data: &[u8]) -> Vec<u8> {
        todo!()
    }

    /// Rejects any digest whose length is not exactly [`Self::digest_len`].
    ///
    /// A signer must never sign bytes of the wrong size: that is how a
    /// truncated or forged "digest" would slip through.
    pub fn check_digest(self, digest: &[u8]) -> Result<(), DigestLengthError> {
        todo!()
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl FromStr for HashAlgorithm {
    type Err = UnknownAlgorithmError;

    /// Accepts `SHA-256`/`SHA256`, `SHA-384`/`SHA384`, `SHA-512`/`SHA512`,
    /// case-insensitively and without trimming.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
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
