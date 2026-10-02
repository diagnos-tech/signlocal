//! Der_to_raw: rejected input.

use super::*;

#[test]
fn rejects_malformed_structure() {
    let cases = [
        ("empty input", ""),
        ("tag only", "30"),
        ("empty sequence", "30 00"),
        ("length only", "30 06"),
        ("sequence without s", "30 03 02 01 01"),
        ("input cut inside s", "30 06 02 01 01 02 01"),
        ("s longer than what remains", "30 06 02 01 01 02 02 01"),
        (
            "outer length larger than the data",
            "30 07 02 01 01 02 01 01",
        ),
        (
            "outer length smaller than the data",
            "30 05 02 01 01 02 01 01",
        ),
        ("r length overruns the sequence", "30 06 02 05 01 02 01 01"),
        ("SET instead of SEQUENCE", "31 06 02 01 01 02 01 01"),
        ("INTEGER instead of SEQUENCE", "02 06 02 01 01 02 01 01"),
        ("r is an OCTET STRING", "30 06 04 01 01 02 01 01"),
        ("s is an OCTET STRING", "30 06 02 01 01 04 01 01"),
        ("r is a BIT STRING", "30 06 03 01 01 02 01 01"),
        ("byte after the sequence", "30 06 02 01 01 02 01 01 00"),
        (
            "second sequence after the first",
            "30 06 02 01 01 02 01 01 30 06 02 01 01 02 01 01",
        ),
        (
            "byte inside the sequence after s",
            "30 07 02 01 01 02 01 01 00",
        ),
        (
            "indefinite sequence length",
            "30 80 02 01 01 02 01 01 00 00",
        ),
        (
            "two-byte long-form sequence length",
            "30 82 00 06 02 01 01 02 01 01",
        ),
        (
            "three-byte long-form sequence length",
            "30 83 00 00 06 02 01 01 02 01 01",
        ),
        ("reserved length octet", "30 ff 02 01 01 02 01 01"),
        ("indefinite integer length", "30 06 02 80 01 02 01 01"),
        (
            "two-byte long-form integer length",
            "30 08 02 82 00 01 01 02 01 01",
        ),
        ("empty r", "30 05 02 00 02 01 01"),
        ("empty s", "30 05 02 01 01 02 00"),
        ("negative r", "30 06 02 01 80 02 01 01"),
        ("negative s", "30 06 02 01 01 02 01 80"),
        ("negative r with ff padding", "30 07 02 02 ff 01 02 01 01"),
        (
            "negative s with 80 00 content",
            "30 07 02 01 01 02 02 80 00",
        ),
        ("r is zero", "30 06 02 01 00 02 01 01"),
        ("s is zero", "30 06 02 01 01 02 01 00"),
        ("r and s are zero", "30 06 02 01 00 02 01 00"),
    ];
    for (label, hex) in cases {
        assert_eq!(
            der_to_raw(&hex_decode(hex), P256),
            Err(Malformed),
            "{label}"
        );
    }
}

#[test]
fn rejects_zero_written_with_padding() {
    for hex in [
        "30 07 02 02 00 00 02 01 01",          // r = 00 00
        "30 08 02 01 01 02 03 00 00 00",       // s = 00 00 00
        "30 0a 02 04 00 00 00 00 02 02 00 00", // both
    ] {
        assert_eq!(der_to_raw(&hex_decode(hex), P256), Err(Malformed), "{hex}");
    }
}

#[test]
fn rejects_integers_wider_than_the_field() {
    let big = |curve: Curve| counting_bytes(1, curve.field_len() + 1);
    for curve in ALL_CURVES {
        let too_big = big(curve);
        let fine = vec![1];
        let cases = [
            ("r too wide", der_ecdsa(&too_big, &fine)),
            ("s too wide", der_ecdsa(&fine, &too_big)),
            ("both too wide", der_ecdsa(&too_big, &too_big)),
            (
                "r too wide behind a sign byte",
                der_ecdsa(
                    &concat(&[&[0], &counting_bytes(0x80, curve.field_len() + 1)]),
                    &fine,
                ),
            ),
            (
                "r too wide behind extra zeros",
                der_ecdsa(&concat(&[&[0, 0], &too_big]), &fine),
            ),
        ];
        for (label, der) in cases {
            assert_eq!(
                der_to_raw(&der, curve),
                Err(IntegerTooLarge),
                "{curve:?}: {label}"
            );
        }
    }
}

#[test]
fn rejects_a_signature_of_a_bigger_curve() {
    for vector in ecdsa_vectors() {
        let smaller: &[Curve] = match vector.curve {
            P256 => &[],
            P384 => &[P256],
            P521 => &[P256, P384],
            other => panic!("no fixture vector for {other:?}"),
        };
        for &curve in smaller {
            assert_eq!(
                der_to_raw(&vector.der, curve),
                Err(IntegerTooLarge),
                "{:?} {} read as {curve:?}",
                vector.curve,
                vector.shape
            );
        }
    }
}

#[test]
fn a_structural_problem_wins_over_an_oversized_integer() {
    // Both integers are checked for shape before either is sized.
    let too_wide = counting_bytes(1, 33);
    let cases = [
        ("s negative", der_ecdsa(&too_wide, &[0x80])),
        ("r negative", der_ecdsa(&[0x80], &too_wide)),
        ("s empty", der_ecdsa(&too_wide, &[])),
        ("s zero", der_ecdsa(&too_wide, &[0x00])),
    ];
    for (label, der) in cases {
        assert_eq!(der_to_raw(&der, P256), Err(Malformed), "{label}");
    }
}
