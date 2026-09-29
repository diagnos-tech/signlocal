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

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

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

    /// Ignores `:` and spaces anywhere; the rest must be 64 hex digits.
    ///
    /// SPEC: only `:` and the ASCII space are ignored; tabs and other
    /// whitespace make the text invalid.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nibbles = s
            .chars()
            .filter(|c| !matches!(c, ':' | ' '))
            .map(|c| c.to_digit(16).and_then(|d| u8::try_from(d).ok()))
            .collect::<Option<Vec<u8>>>()
            .ok_or(ParseFingerprintError)?;
        if nibbles.len() != 64 {
            return Err(ParseFingerprintError);
        }

        let mut bytes = [0u8; 32];
        for (byte, [high, low]) in bytes.iter_mut().zip(nibbles.as_chunks::<2>().0) {
            *byte = (high << 4) | low;
        }
        Ok(Self(bytes))
    }
}

/// Text that is not a SHA-256 fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a SHA-256 fingerprint")]
pub struct ParseFingerprintError;

#[cfg(test)]
mod tests {
    use super::*;

    const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn of_is_sha256_of_the_der() {
        let fp = Fingerprint::of(b"abc");
        assert_eq!(fp.to_hex(), ABC_SHA256);
        assert_eq!(fp.as_bytes().len(), 32);
    }

    #[test]
    fn display_is_uppercase_pairs_with_colons() {
        let text = Fingerprint::of(b"abc").to_string();
        assert_eq!(text.len(), 95);
        assert!(text.starts_with("BA:78:16:BF:"));
        assert!(text.ends_with(":F2:00:15:AD"));
    }

    #[test]
    fn debug_wraps_the_lowercase_hex() {
        assert_eq!(
            format!("{:?}", Fingerprint::of(b"abc")),
            format!("Fingerprint({ABC_SHA256})")
        );
    }

    #[test]
    fn parses_every_rendering_it_prints() {
        let fp = Fingerprint::of(b"abc");
        assert_eq!(fp.to_string().parse(), Ok(fp));
        assert_eq!(fp.to_hex().parse(), Ok(fp));
        assert_eq!(fp.to_hex().to_uppercase().parse(), Ok(fp));
        assert_eq!(format!(" {} : ", fp.to_hex()).parse(), Ok(fp));
    }

    #[test]
    fn rejects_bad_length_and_non_hex() {
        let good = ABC_SHA256;
        let too_short = &good[..62];
        let too_long = format!("{good}00");
        let non_hex = good.replacen('b', "g", 1);
        let tab = format!("\t{good}");
        let wide_digit = good.replacen('b', "\u{ff22}", 1);
        for bad in [
            "",
            too_short,
            too_long.as_str(),
            non_hex.as_str(),
            tab.as_str(),
            wide_digit.as_str(),
        ] {
            assert_eq!(
                bad.parse::<Fingerprint>(),
                Err(ParseFingerprintError),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn from_bytes_round_trips() {
        let fp = Fingerprint::from_bytes([7; 32]);
        assert_eq!(fp.as_bytes(), &[7; 32]);
    }
}
