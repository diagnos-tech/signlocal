//! Display and Debug.

use super::*;

#[test]
fn displays_uppercase_pairs_separated_by_colons() {
    let fp = Fingerprint::from_bytes(bytes_32(0));
    assert_eq!(fp.to_string(), counting_display());
    assert_eq!(
        Fingerprint::of(b"abc").to_string(),
        "BA:78:16:BF:8F:01:CF:EA:41:41:40:DE:5D:AE:22:23:B0:03:61:A3:96:17:7A:9C:B4:10:FF:61:F2:00:15:AD"
    );
}

#[test]
fn display_is_95_characters_with_31_separators() {
    for start in [0u8, 0x7F, 0xFF] {
        let text = Fingerprint::from_bytes(bytes_32(start)).to_string();
        assert_eq!(text.len(), 95);
        assert_eq!(text.matches(':').count(), 31);
        assert!(!text.chars().any(|c| c.is_ascii_lowercase()));
        assert!(text.split(':').all(|pair| pair.len() == 2));
    }
}

#[test]
fn display_zero_pads_small_bytes() {
    let text = Fingerprint::from_bytes([0x0A; 32]).to_string();
    assert!(text.starts_with("0A:0A:"));
    assert!(text.ends_with(":0A"));
}

#[test]
fn debug_wraps_the_lowercase_hex() {
    let fp = Fingerprint::of(b"abc");
    assert_eq!(format!("{fp:?}"), format!("Fingerprint({ABC_HEX})"));
    assert_eq!(
        format!("{:?}", Fingerprint::from_bytes([0; 32])),
        format!("Fingerprint({})", "0".repeat(64))
    );
}
