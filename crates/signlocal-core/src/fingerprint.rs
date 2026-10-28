//! Certificate identity: the SHA-256 of its DER encoding.

use std::fmt::{self, Write};
use std::str::FromStr;

use sha2::{Digest, Sha256};

use crate::hex;

/// SHA-256 of a certificate's DER bytes.
///
/// The one identity used across the project: de-duplication between key
/// sources, "last used certificate" per site and selection on the command line.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fingerprint([u8; 32]);

impl Fingerprint {
    /// Fingerprint of a DER-encoded certificate.
    pub fn of(der: &[u8]) -> Self {
        Self(Sha256::digest(der).into())
    }

    /// A fingerprint computed elsewhere, such as one stored as "last used".
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The raw SHA-256, for comparing with hashes key stores report.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// 64 lowercase hex digits, no separators.
    pub fn to_hex(&self) -> String {
        hex::lower(&self.0)
    }
}

/// Uppercase hex pairs separated by `:`, the way OS certificate viewers show it.
impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_char(':')?;
            }
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fingerprint({})", self.to_hex())
    }
}

impl FromStr for Fingerprint {
    type Err = ParseFingerprintError;

    /// Ignores `:` and the ASCII space anywhere; the rest must be exactly 64
    /// hex digits. Other whitespace is refused so that text split across
    /// lines or pasted with tabs is noticed rather than silently joined.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut nibbles = s
            .chars()
            .filter(|c| !matches!(c, ':' | ' '))
            .map(|c| c.to_digit(16).and_then(|d| u8::try_from(d).ok()));
        let mut next = || nibbles.next().flatten().ok_or(ParseFingerprintError);

        let mut bytes = [0u8; 32];
        for byte in &mut bytes {
            *byte = (next()? << 4) | next()?;
        }
        match nibbles.next() {
            None => Ok(Self(bytes)),
            Some(_) => Err(ParseFingerprintError),
        }
    }
}

/// Text that is not a SHA-256 fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a SHA-256 fingerprint")]
pub struct ParseFingerprintError;
