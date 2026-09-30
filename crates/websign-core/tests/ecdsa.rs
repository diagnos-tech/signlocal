//! SPEC §4: `Curve`, `der_to_raw`, `raw_to_der`.

mod common;

use std::collections::HashSet;

use common::{
    Rng, counting_bytes, der_ecdsa, ecdsa_vectors, hex_decode, left_pad, parse_strict_ecdsa_der,
};
use websign_core::ecdsa::{Curve, EcdsaEncodingError, der_to_raw, raw_to_der};

use Curve::{P256, P384, P521};
use EcdsaEncodingError::{IntegerTooLarge, Malformed, WrongLength};

const ALL_CURVES: [Curve; 3] = [P256, P384, P521];

/// `r || s` from big-endian magnitudes, each left-padded to the field size.
fn raw_of(curve: Curve, r: &[u8], s: &[u8]) -> Vec<u8> {
    let mut out = left_pad(r, curve.field_len());
    out.extend(left_pad(s, curve.field_len()));
    out
}

fn zeros(count: usize) -> Vec<u8> {
    vec![0; count]
}

fn concat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

// --- Curve -----------------------------------------------------------------

#[test]
fn reports_field_length_in_bytes() {
    assert_eq!(P256.field_len(), 32);
    assert_eq!(P384.field_len(), 48);
    assert_eq!(P521.field_len(), 66);
}

#[test]
fn reports_raw_signature_length_as_twice_the_field() {
    assert_eq!(P256.signature_len(), 64);
    assert_eq!(P384.signature_len(), 96);
    assert_eq!(P521.signature_len(), 132);
}

#[test]
fn reports_nist_names() {
    assert_eq!(P256.name(), "P-256");
    assert_eq!(P384.name(), "P-384");
    assert_eq!(P521.name(), "P-521");
}

#[test]
fn curve_is_copy_eq_and_hashable() {
    let curve = P256;
    let copy = curve;
    assert_eq!(curve, copy);
    assert_ne!(P256, P384);
    assert_eq!(ALL_CURVES.into_iter().collect::<HashSet<_>>().len(), 3);
}

#[test]
fn maps_named_curve_oids() {
    assert_eq!(Curve::from_oid("1.2.840.10045.3.1.7"), Some(P256));
    assert_eq!(Curve::from_oid("1.3.132.0.34"), Some(P384));
    assert_eq!(Curve::from_oid("1.3.132.0.35"), Some(P521));
}

#[test]
fn rejects_other_oids_and_look_alikes() {
    let unknown = [
        "",
        " ",
        "1.2.840.10045.3.1.7 ",
        " 1.3.132.0.34",
        "1.3.132.0.33", // secp224r1
        "1.3.132.0.10", // secp256k1
        "1.3.132.0.36", // sect283k1 neighbour
        "1.3.132.0.3",
        "1.3.132.0.340",
        "1.3.132.0.34.1",
        "1.3.132.0.35.0",
        "1.2.840.10045.3.1.1", // prime192v1
        "1.2.840.10045.3.1.70",
        "1.2.840.10045.2.1", // id-ecPublicKey, not a curve
        "1.3.101.112",       // Ed25519
        "prime256v1",
        "secp384r1",
        "P-256",
        "P-521",
        "2.16.840.1.101.3.4.2.1",
        "not an oid",
    ];
    for oid in unknown {
        assert_eq!(Curve::from_oid(oid), None, "{oid:?}");
    }
}

#[test]
fn formats_encoding_errors() {
    assert_eq!(Malformed.to_string(), "malformed ECDSA signature");
    assert_eq!(
        IntegerTooLarge.to_string(),
        "ECDSA integer does not fit the curve"
    );
    assert_eq!(
        WrongLength {
            expected: 64,
            actual: 63
        }
        .to_string(),
        "raw ECDSA signature must be 64 bytes, got 63"
    );
}

#[test]
fn encoding_error_is_a_cloneable_comparable_std_error() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<EcdsaEncodingError>();
    assert_ne!(Malformed, IntegerTooLarge);
}

// --- der_to_raw: valid input -------------------------------------------------

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

// --- der_to_raw: rejected input ----------------------------------------------

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

// --- raw_to_der ----------------------------------------------------------------

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

// --- round trip and OpenSSL vectors -----------------------------------------------

#[test]
fn zero_halves_encode_but_do_not_decode_back() {
    for curve in ALL_CURVES {
        let der = raw_to_der(&zeros(curve.signature_len()), curve).unwrap();
        assert_eq!(der, hex_decode("30 06 02 01 00 02 01 00"));
        assert_eq!(der_to_raw(&der, curve), Err(Malformed));
    }
}

#[test]
fn round_trips_extreme_values() {
    for curve in ALL_CURVES {
        let len = curve.field_len();
        let one = left_pad(&[1], len);
        let patterns: Vec<Vec<u8>> = vec![
            one.clone(),
            vec![0xFF; len],
            vec![0x80; len],
            vec![0x7F; len],
            concat(&[&[0x80], &zeros(len - 1)]),
            concat(&[&zeros(len - 1), &[0x80]]),
        ];
        for r in &patterns {
            for s in &patterns {
                let raw = concat(&[r, s]);
                let der = raw_to_der(&raw, curve).unwrap();
                assert_eq!(der_to_raw(&der, curve), Ok(raw), "{curve:?}");
            }
        }
    }
}

#[test]
fn round_trips_random_values_and_always_emits_strict_minimal_der() {
    let mut rng = Rng::new(0x5EED_0001);
    for curve in ALL_CURVES {
        let len = curve.field_len();
        for _ in 0..1500 {
            let half = |rng: &mut Rng| {
                let mut value = rng.bytes(len);
                let zeroed = [0, 0, 0, 1, 2, 3][rng.below(6) as usize].min(len - 1);
                value[..zeroed].fill(0);
                if value.iter().all(|&b| b == 0) {
                    value[len - 1] = 1;
                }
                value
            };
            let raw = concat(&[&half(&mut rng), &half(&mut rng)]);
            let der = raw_to_der(&raw, curve).unwrap();
            let (r_int, s_int) = parse_strict_ecdsa_der(&der)
                .unwrap_or_else(|| panic!("{curve:?}: not strict minimal DER: {der:02x?}"));
            assert_eq!(strip_zeros(&r_int), strip_zeros(&raw[..len]), "{curve:?} r");
            assert_eq!(strip_zeros(&s_int), strip_zeros(&raw[len..]), "{curve:?} s");
            assert_eq!(der_to_raw(&der, curve), Ok(raw), "{curve:?}");
        }
    }
}

fn strip_zeros(bytes: &[u8]) -> &[u8] {
    let first = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    &bytes[first..]
}

#[test]
fn openssl_fixtures_cover_every_shape_on_every_curve() {
    let vectors = ecdsa_vectors();
    for curve in ALL_CURVES {
        let shapes: HashSet<&str> = vectors
            .iter()
            .filter(|v| v.curve == curve)
            .map(|v| v.shape.as_str())
            .collect();
        for shape in [
            "plain",
            "high-r",
            "high-s",
            "high-both",
            "short-r",
            "short-s",
        ] {
            assert!(
                shapes.contains(shape),
                "fixture lacks {shape} for {curve:?}"
            );
        }
    }
}

#[test]
fn openssl_fixture_shapes_are_what_their_labels_say() {
    for v in ecdsa_vectors() {
        let len = v.curve.field_len();
        let (r, s) = (&v.raw[..len], &v.raw[len..]);
        let high = |half: &[u8]| strip_zeros(half)[0] >= 0x80;
        match v.shape.as_str() {
            "short-r" => assert_eq!(r[0], 0, "{v:?}"),
            "short-s" => assert_eq!(s[0], 0, "{v:?}"),
            "high-r" => assert!(high(r), "{v:?}"),
            "high-s" => assert!(high(s), "{v:?}"),
            "high-both" => assert!(high(r) && high(s), "{v:?}"),
            "plain" => assert!(
                (1..0x80).contains(&r[0]) && (1..0x80).contains(&s[0]),
                "{v:?}"
            ),
            other => panic!("unknown shape {other}"),
        }
    }
}

#[test]
fn converts_openssl_der_to_the_raw_form_openssl_readback_gives() {
    for v in ecdsa_vectors() {
        assert_eq!(
            der_to_raw(&v.der, v.curve),
            Ok(v.raw.clone()),
            "{:?} {}",
            v.curve,
            v.shape
        );
    }
}

#[test]
fn converts_raw_to_the_der_openssl_produced() {
    for v in ecdsa_vectors() {
        assert_eq!(
            raw_to_der(&v.raw, v.curve),
            Ok(v.der.clone()),
            "{:?} {}",
            v.curve,
            v.shape
        );
    }
}

#[test]
fn p521_openssl_signatures_use_the_long_length_form_when_long_enough() {
    let long: Vec<_> = ecdsa_vectors()
        .into_iter()
        .filter(|v| v.curve == P521 && v.der.len() > 130)
        .collect();
    assert!(
        !long.is_empty(),
        "fixtures should include a P-521 signature over 127 bytes"
    );
    for v in long {
        assert_eq!(v.der[..2], [0x30, 0x81], "{}", v.shape);
        assert_eq!(der_to_raw(&v.der, P521), Ok(v.raw), "{}", v.shape);
    }
}

#[test]
fn arbitrary_bytes_are_an_error_or_a_value_never_a_panic() {
    let mut rng = Rng::new(0x5EED_0002);
    for _ in 0..3000 {
        let len = rng.below(160) as usize;
        let mut bytes = rng.bytes(len);
        // Mostly start like a signature so the parser gets past the first tag.
        if let Some(first) = bytes.first_mut() {
            *first = 0x30;
        }
        if len > 2 && rng.below(2) == 0 {
            bytes[2] = 0x02;
        }
        for curve in ALL_CURVES {
            let _ = der_to_raw(&bytes, curve);
            let _ = raw_to_der(&bytes, curve);
        }
    }
}
