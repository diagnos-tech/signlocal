//! The primitive values certificates carry flags and keys in.

use super::DerError;

/// BOOLEAN content. Any non-zero octet is TRUE, as BER allows; DER's `ff`
/// is only the canonical spelling.
pub(in crate::cert) fn boolean(content: &[u8]) -> Result<bool, DerError> {
    match content {
        [value] => Ok(*value != 0),
        _ => Err(DerError::Invalid("BOOLEAN")),
    }
}

/// BIT STRING content split into its bytes; the unused trailing bits of the
/// last byte are left for the caller to ignore.
pub(in crate::cert) fn bit_string(content: &[u8]) -> Result<BitString<'_>, DerError> {
    match content.split_first() {
        Some((&unused, bytes)) if unused <= 7 && (unused == 0 || !bytes.is_empty()) => {
            Ok(BitString { unused, bytes })
        }
        _ => Err(DerError::Invalid("BIT STRING")),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::cert) struct BitString<'a> {
    pub(in crate::cert) unused: u8,
    pub(in crate::cert) bytes: &'a [u8],
}

impl<'a> BitString<'a> {
    /// Bit `position`, counted from the most significant bit of the first
    /// byte as X.690 numbers them; bits past the end read as zero.
    pub(in crate::cert) fn bit(&self, position: usize) -> bool {
        self.bytes
            .get(position / 8)
            .is_some_and(|byte| byte & (0x80 >> (position % 8)) != 0)
    }

    /// The bytes, if the string is a whole number of them (keys are).
    pub(in crate::cert) fn whole_bytes(&self) -> Option<&'a [u8]> {
        (self.unused == 0).then_some(self.bytes)
    }
}
