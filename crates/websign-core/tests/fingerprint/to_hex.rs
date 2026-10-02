//! To_hex.

use super::*;

#[test]
fn hex_is_64_lowercase_digits_without_separators() {
    let hex = Fingerprint::from_bytes([0xAB; 32]).to_hex();
    assert_eq!(hex, "ab".repeat(32));
    assert_eq!(hex.len(), 64);
    let hex = Fingerprint::from_bytes(bytes_32(0xF0)).to_hex();
    assert!(
        hex.chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    );
}

#[test]
fn hex_keeps_leading_zeros() {
    assert_eq!(Fingerprint::from_bytes([0; 32]).to_hex(), "0".repeat(64));
    let mut bytes = [0; 32];
    bytes[31] = 1;
    assert_eq!(
        Fingerprint::from_bytes(bytes).to_hex(),
        format!("{}01", "00".repeat(31))
    );
}
