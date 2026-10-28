//! Raw_to_der.

use super::*;

#[test]
fn rejects_raw_signatures_of_the_wrong_length() {
    for curve in ALL_CURVES {
        let expected = curve.signature_len();
        for actual in [0, 1, 32, 63, 64, 65, 95, 96, 97, 131, 132, 133, 200] {
            let result = raw_to_der(&vec![0x11; actual], curve);
            if actual == expected {
                assert!(result.is_ok(), "{curve:?}/{actual}");
            } else {
                assert_eq!(
                    result,
                    Err(WrongLength { expected, actual }),
                    "{curve:?}/{actual}"
                );
            }
        }
    }
}

#[test]
fn encodes_small_values_minimally() {
    let cases: [(&[u8], &[u8], &str); 8] = [
        (&[1], &[1], "30 06 02 01 01 02 01 01"),
        (&[0x7F], &[0x80], "30 07 02 01 7f 02 02 00 80"),
        (&[0x80], &[0xFF], "30 08 02 02 00 80 02 02 00 ff"),
        (&[1, 0], &[0x7F, 0xFF], "30 08 02 02 01 00 02 02 7f ff"),
        (&[0, 0xFF], &[0, 0, 0x12], "30 07 02 02 00 ff 02 01 12"),
        (&[0, 0, 0x80, 0], &[0x7F], "30 08 02 03 00 80 00 02 01 7f"),
        (&[0], &[0], "30 06 02 01 00 02 01 00"),
        (&[0], &[1], "30 06 02 01 00 02 01 01"),
    ];
    for (r, s, expected) in cases {
        for curve in ALL_CURVES {
            assert_eq!(
                raw_to_der(&raw_of(curve, r, s), curve),
                Ok(hex_decode(expected)),
                "{curve:?} r={r:02x?} s={s:02x?}"
            );
        }
    }
}

#[test]
fn keeps_full_size_values_and_adds_a_sign_byte_only_when_needed() {
    for curve in [P256, P384] {
        let len = curve.field_len();
        let (low, high) = (counting_bytes(0x11, len), counting_bytes(0x80, len));
        assert_eq!(
            raw_to_der(&concat(&[&low, &low]), curve),
            Ok(der_ecdsa(&low, &low)),
            "{curve:?} low/low"
        );
        assert_eq!(
            raw_to_der(&concat(&[&high, &low]), curve),
            Ok(der_ecdsa(&concat(&[&[0], &high]), &low)),
            "{curve:?} high/low"
        );
        assert_eq!(
            raw_to_der(&concat(&[&low, &high]), curve),
            Ok(der_ecdsa(&low, &concat(&[&[0], &high]))),
            "{curve:?} low/high"
        );
        assert_eq!(
            raw_to_der(&concat(&[&high, &high]), curve),
            Ok(der_ecdsa(&concat(&[&[0], &high]), &concat(&[&[0], &high]))),
            "{curve:?} high/high"
        );
    }
}

#[test]
fn encodes_the_largest_values() {
    let der = raw_to_der(&[0xFF; 64], P256).unwrap();
    let expected = der_ecdsa(&concat(&[&[0], &[0xFF; 32]]), &concat(&[&[0], &[0xFF; 32]]));
    assert_eq!(der, expected);
    assert_eq!(der[..2], [0x30, 0x46]);
}

#[test]
fn strips_every_leading_zero_but_keeps_one_byte() {
    let mut raw = zeros(62);
    raw.extend([0x12, 0x34]);
    assert_eq!(
        raw_to_der(&raw, P256),
        Ok(hex_decode("30 07 02 01 00 02 02 12 34"))
    );
    let mut raw = zeros(63);
    raw.push(0x01);
    assert_eq!(
        raw_to_der(&raw, P256),
        Ok(hex_decode("30 06 02 01 00 02 01 01"))
    );
}

#[test]
fn uses_the_long_length_form_only_above_127_bytes() {
    for (r_len, s_len, expected_head) in [
        (61, 62, vec![0x30, 0x7F]),
        (62, 62, vec![0x30, 0x81, 0x80]),
        (62, 63, vec![0x30, 0x81, 0x81]),
    ] {
        let (r, s) = (counting_bytes(0x11, r_len), counting_bytes(0x21, s_len));
        let der = raw_to_der(&raw_of(P521, &r, &s), P521).unwrap();
        assert_eq!(
            der[..expected_head.len()],
            expected_head[..],
            "{r_len}/{s_len}"
        );
        assert_eq!(der, der_ecdsa(&r, &s), "{r_len}/{s_len}");
    }
}

#[test]
fn encodes_full_size_p521_values_in_long_form() {
    let (r, s) = (counting_bytes(1, 66), counting_bytes(2, 66));
    let der = raw_to_der(&concat(&[&r, &s]), P521).unwrap();
    assert_eq!(der[..3], [0x30, 0x81, 0x88]);
    assert_eq!(der, der_ecdsa(&r, &s));
    assert_eq!(der.len(), 139);
}
