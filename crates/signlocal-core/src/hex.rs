//! Lowercase hex encoding, the only text form of binary identifiers that the
//! crate needs. Kept local so the crate does not pull in a dependency for it.

use std::fmt::Write;

/// Encodes `bytes` as lowercase hex digits, two per byte.
pub(crate) fn lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // Writing to a `String` cannot fail.
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::lower;

    #[test]
    fn encodes_lowercase_with_leading_zeros() {
        assert_eq!(lower(&[0x00, 0x0a, 0xff, 0x80]), "000aff80");
    }

    #[test]
    fn empty_input_is_empty_text() {
        assert_eq!(lower(&[]), "");
    }
}
