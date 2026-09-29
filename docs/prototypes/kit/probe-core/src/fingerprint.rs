//! Certificate identity: the SHA-256 of its DER encoding.

use std::fmt;
use std::str::FromStr;

/// SHA-256 of a certificate's DER bytes.
///
/// The one identity used across the project: de-duplication between key
/// sources, "last used certificate" per site and selection on the command line.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fingerprint([u8; 32]);

impl Fingerprint {
    /// Fingerprint of a DER-encoded certificate.
    pub fn of(der: &[u8]) -> Self {
        todo!()
    }

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// 64 lowercase hex digits, no separators.
    pub fn to_hex(&self) -> String {
        todo!()
    }
}

/// Uppercase hex pairs separated by `:`, the way OS certificate viewers show it.
impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl FromStr for Fingerprint {
    type Err = ParseFingerprintError;

    /// Ignores `:` and spaces anywhere; the rest must be 64 hex digits.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

/// Text that is not a SHA-256 fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a SHA-256 fingerprint")]
pub struct ParseFingerprintError;
