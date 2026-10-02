use super::*;

const EXAMPLE: &str = "3B:D5:18:FF:81:91:FE:1F:C3:..:..:..:..:..:..";

#[test]
fn historical_bytes_and_tck_are_masked_and_interface_bytes_kept() {
    assert_eq!(
        mask_atr("3B:D5:18:FF:81:91:FE:1F:C3:80:73:C8:21:10:0A"),
        EXAMPLE
    );
}

#[test]
fn any_hex_spelling_gives_the_same_text() {
    for spelling in [
        "3BD518FF8191FE1FC38073C821100A",
        "3b d5 18 ff 81 91 fe 1f c3 80 73 c8 21 10 0a",
    ] {
        assert_eq!(mask_atr(spelling), EXAMPLE, "{spelling}");
    }
}

#[test]
fn t0_only_atr_has_no_tck() {
    // K = 2, no interface bytes, protocol T=0 implied.
    assert_eq!(mask_atr("3B02AABB"), "3B:02:..:..");
    assert_eq!(mask_atr("3B00"), "3B:00");
    assert_eq!(mask_atr("3F00"), "3F:00");
}

#[test]
fn explicit_t0_in_td1_needs_no_tck_but_t1_does() {
    assert_eq!(mask_atr("3B8100AA"), "3B:81:00:..");
    assert_eq!(mask_atr("3B8101AABB"), "3B:81:01:..:..");
    assert_eq!(mask_atr("3B8101AA"), "4 bytes");
}

#[test]
fn a_serial_number_in_the_historical_bytes_never_appears() {
    let text = mask_atr("3B8F8001804F0CA0000003060300030000000068");
    assert!(!text.contains("A0") && !text.contains("68"), "{text}");
}

#[test]
fn truncated_or_padded_atrs_print_only_their_length() {
    assert_eq!(mask_atr("3BD518FF8191FE1FC38073C8"), "12 bytes");
    assert_eq!(mask_atr("3BD518FF8191FE1FC38073C821100A00"), "16 bytes");
    assert_eq!(mask_atr("3B"), "1 byte");
    assert_eq!(mask_atr("3BFF"), "2 bytes");
    assert_eq!(mask_atr("00112233"), "4 bytes");
}

#[test]
fn text_that_is_not_hex_bytes_is_unreadable() {
    for garbage in ["", "  ", "3BD", "not an atr", "3B:ZZ", "é3", "🦀🦀"] {
        assert_eq!(mask_atr(garbage), "unreadable", "{garbage:?}");
    }
}

#[test]
fn no_input_makes_it_panic() {
    let long = "FF".repeat(4096);
    let chains = "3BFF".to_owned() + &"81".repeat(600);
    for input in [long.as_str(), chains.as_str(), "3B8F", "3BF0", "3BFFFFFFFF"] {
        let _ = mask_atr(input);
    }
    for first in 0..=255u8 {
        for second in 0..=255u8 {
            let _ = mask_atr(&format!("{first:02X}{second:02X}"));
            let _ = mask_atr(&format!("3B{first:02X}{second:02X}81{first:02X}"));
        }
    }
}
