//! Der_to_raw: valid input.

use super::*;

#[test]
fn pads_one_byte_integers_to_the_field_size() {
    let der = hex_decode("30 06 02 01 01 02 01 02");
    for curve in ALL_CURVES {
        assert_eq!(
            der_to_raw(&der, curve),
            Ok(raw_of(curve, &[1], &[2])),
            "{curve:?}"
        );
    }
}

#[test]
fn keeps_full_size_integers_as_they_are() {
    for curve in ALL_CURVES {
        let (r, s) = (
            counting_bytes(1, curve.field_len()),
            counting_bytes(0x41, curve.field_len()),
        );
        let der = der_ecdsa(&r, &s);
        assert_eq!(der_to_raw(&der, curve), Ok(concat(&[&r, &s])), "{curve:?}");
    }
}

#[test]
fn drops_the_sign_byte_of_integers_with_the_top_bit_set() {
    for curve in [P256, P384] {
        let len = curve.field_len();
        let (r, s) = (counting_bytes(0x80, len), counting_bytes(0xC0, len));
        let der = der_ecdsa(&concat(&[&[0], &r]), &concat(&[&[0], &s]));
        assert_eq!(der_to_raw(&der, curve), Ok(concat(&[&r, &s])), "{curve:?}");
    }
}

#[test]
fn pads_short_integers_on_the_left() {
    let r = counting_bytes(0x11, 31);
    let s = counting_bytes(0x21, 30);
    let der = der_ecdsa(&r, &s);
    let raw = der_to_raw(&der, P256).unwrap();
    assert_eq!(raw.len(), 64);
    assert_eq!(raw[..32], concat(&[&[0], &r])[..]);
    assert_eq!(raw[32..], concat(&[&[0, 0], &s])[..]);
}

#[test]
fn converts_p521_signatures_that_need_the_long_length_form() {
    let (r, s) = (counting_bytes(1, 66), counting_bytes(2, 66));
    let der = der_ecdsa(&r, &s);
    assert_eq!(
        der[..3],
        [0x30, 0x81, 0x88],
        "test input uses the long form"
    );
    assert_eq!(der_to_raw(&der, P521), Ok(concat(&[&r, &s])));
}

#[test]
fn converts_p521_signatures_whose_integers_lost_their_leading_zero_byte() {
    let (r, s) = (counting_bytes(0x81, 65), counting_bytes(0x11, 64));
    let der = der_ecdsa(&concat(&[&[0], &r]), &s);
    assert_eq!(der_to_raw(&der, P521), Ok(raw_of(P521, &r, &s)));
}

#[test]
fn switches_between_short_and_long_sequence_length_at_128() {
    // SEQUENCE body = 4 header bytes + both integer contents.
    for (r_len, s_len, expected_head) in [
        (61, 62, vec![0x30, 0x7F]),
        (62, 62, vec![0x30, 0x81, 0x80]),
        (63, 62, vec![0x30, 0x81, 0x81]),
    ] {
        let (r, s) = (counting_bytes(0x11, r_len), counting_bytes(0x21, s_len));
        let der = der_ecdsa(&r, &s);
        assert_eq!(
            der[..expected_head.len()],
            expected_head[..],
            "test input framing"
        );
        assert_eq!(
            der_to_raw(&der, P521),
            Ok(raw_of(P521, &r, &s)),
            "{r_len}/{s_len}"
        );
    }
}

#[test]
fn tolerates_redundant_leading_zeros() {
    let value = counting_bytes(0x11, 32);
    let high = counting_bytes(0x80, 32);
    /// INTEGER contents as encoded (with redundant zeros) and the values they mean.
    struct Padded {
        r_encoded: Vec<u8>,
        s_encoded: Vec<u8>,
        r: Vec<u8>,
        s: Vec<u8>,
    }
    let padded = |r_encoded: Vec<u8>, s_encoded: Vec<u8>, r: Vec<u8>, s: Vec<u8>| Padded {
        r_encoded,
        s_encoded,
        r,
        s,
    };
    let cases = [
        padded(vec![0, 0, 5], vec![1], vec![5], vec![1]),
        padded(
            vec![0, 0x11, 0x22],
            vec![0, 0, 0, 7],
            vec![0x11, 0x22],
            vec![7],
        ),
        padded(
            vec![0, 0, 0x80, 0xAA],
            vec![0, 0x7F],
            vec![0x80, 0xAA],
            vec![0x7F],
        ),
        padded(
            concat(&[&[0, 0], &value]),
            concat(&[&[0], &value]),
            value.clone(),
            value.clone(),
        ),
        padded(
            concat(&[&[0, 0, 0], &high]),
            concat(&[&[0, 0], &high]),
            high.clone(),
            high.clone(),
        ),
        padded(
            concat(&[&zeros(34), &value]),
            vec![1],
            value.clone(),
            vec![1],
        ),
    ];
    for (i, case) in cases.iter().enumerate() {
        let der = der_ecdsa(&case.r_encoded, &case.s_encoded);
        assert_eq!(
            der_to_raw(&der, P256),
            Ok(raw_of(P256, &case.r, &case.s)),
            "case {i}"
        );
    }
}

#[test]
fn tolerates_padding_that_pushes_the_length_into_long_form() {
    let (r, s) = (counting_bytes(0x11, 32), counting_bytes(0x21, 32));
    let der = der_ecdsa(&concat(&[&zeros(34), &r]), &concat(&[&zeros(34), &s]));
    assert_eq!(
        der[..3],
        [0x30, 0x81, 0x88],
        "test input uses the long form"
    );
    assert_eq!(der_to_raw(&der, P256), Ok(concat(&[&r, &s])));
}

#[test]
fn accepts_a_long_form_integer_length() {
    let (r, s) = (counting_bytes(0x11, 32), counting_bytes(0x21, 32));
    let der = der_ecdsa(&concat(&[&zeros(98), &r]), &s);
    // 30 81 <len> then the first INTEGER: 02 81 82 (130 content bytes) 00 ...
    assert_eq!(
        der[3..7],
        [0x02, 0x81, 0x82, 0],
        "test input has a 130-byte INTEGER"
    );
    assert_eq!(der_to_raw(&der, P256), Ok(concat(&[&r, &s])));
}

#[test]
fn accepts_a_long_form_length_that_the_short_form_could_express() {
    // Tokens are sloppy about minimal lengths; `81 06` means 6 like `06`.
    for hex in [
        "30 81 06 02 01 01 02 01 02",
        "30 07 02 81 01 01 02 01 02",
        "30 81 07 02 81 01 01 02 01 02",
    ] {
        assert_eq!(
            der_to_raw(&hex_decode(hex), P256),
            Ok(raw_of(P256, &[1], &[2])),
            "{hex}"
        );
    }
}

#[test]
fn reads_a_p256_signature_as_p384_by_padding_it() {
    let (r, s) = (counting_bytes(1, 32), counting_bytes(0x41, 32));
    let der = der_ecdsa(&r, &s);
    assert_eq!(der_to_raw(&der, P384), Ok(raw_of(P384, &r, &s)));
}
