use super::*;

const ALL: [Curve; 3] = [Curve::P256, Curve::P384, Curve::P521];

/// `30 len 02 len r 02 len s` from already-encoded INTEGER contents.
fn der(r: &[u8], s: &[u8]) -> Vec<u8> {
    let mut body = vec![0x02];
    push_length(&mut body, r.len());
    body.extend_from_slice(r);
    body.push(0x02);
    push_length(&mut body, s.len());
    body.extend_from_slice(s);
    let mut out = vec![0x30];
    push_length(&mut out, body.len());
    out.extend(body);
    out
}

fn raw_of(curve: Curve, r_tail: &[u8], s_tail: &[u8]) -> Vec<u8> {
    let mut raw = vec![0; curve.signature_len()];
    let n = curve.field_len();
    raw[n - r_tail.len()..n].copy_from_slice(r_tail);
    raw[2 * n - s_tail.len()..].copy_from_slice(s_tail);
    raw
}

#[test]
fn curve_metadata_follows_the_spec() {
    let table = [
        (Curve::P256, 32, 64, "P-256"),
        (Curve::P384, 48, 96, "P-384"),
        (Curve::P521, 66, 132, "P-521"),
    ];
    for (curve, field, sig, name) in table {
        assert_eq!(curve.field_len(), field);
        assert_eq!(curve.signature_len(), sig);
        assert_eq!(curve.name(), name);
    }
}

#[test]
fn curves_are_found_by_oid() {
    assert_eq!(Curve::from_oid("1.2.840.10045.3.1.7"), Some(Curve::P256));
    assert_eq!(Curve::from_oid("1.3.132.0.34"), Some(Curve::P384));
    assert_eq!(Curve::from_oid("1.3.132.0.35"), Some(Curve::P521));
    for other in [
        "",
        "1.3.132.0.10",
        "1.3.132.0.33",
        "1.2.840.10045.2.1",
        "P-256",
    ] {
        assert_eq!(Curve::from_oid(other), None, "{other}");
    }
}

#[test]
fn converts_a_small_signature_to_padded_raw() {
    let raw = der_to_raw(&der(&[0x01], &[0x7f]), Curve::P256).unwrap();
    assert_eq!(raw, raw_of(Curve::P256, &[0x01], &[0x7f]));
}

#[test]
fn drops_the_sign_byte_of_high_integers() {
    let raw = der_to_raw(&der(&[0x00, 0x80], &[0x00, 0xff, 0x01]), Curve::P256).unwrap();
    assert_eq!(raw, raw_of(Curve::P256, &[0x80], &[0xff, 0x01]));
}

#[test]
fn tolerates_extra_leading_zeros() {
    let raw = der_to_raw(
        &der(&[0x00, 0x00, 0x00, 0x05], &[0x00, 0x00, 0x07]),
        Curve::P384,
    )
    .unwrap();
    assert_eq!(raw, raw_of(Curve::P384, &[0x05], &[0x07]));
}

#[test]
fn accepts_a_full_width_integer_and_rejects_one_byte_more() {
    for curve in ALL {
        let full = vec![0x7f; curve.field_len()];
        assert!(der_to_raw(&der(&full, &[1]), curve).is_ok());

        let mut too_big = vec![0x00, 0x01];
        too_big.extend(vec![0xaa; curve.field_len()]);
        assert_eq!(
            der_to_raw(&der(&too_big, &[1]), curve),
            Err(EcdsaEncodingError::IntegerTooLarge)
        );
        assert_eq!(
            der_to_raw(&der(&[1], &too_big), curve),
            Err(EcdsaEncodingError::IntegerTooLarge)
        );
    }
}

#[test]
fn p521_signatures_use_the_long_length_form() {
    let r = vec![0x81; 66];
    let s = vec![0x82; 66];
    let mut r_der = vec![0x00];
    r_der.extend(&r);
    let mut s_der = vec![0x00];
    s_der.extend(&s);
    let encoded = der(&r_der, &s_der);
    assert_eq!(&encoded[..3], &[0x30, 0x81, 0x8a]);

    let raw = der_to_raw(&encoded, Curve::P521).unwrap();
    assert_eq!(&raw[..66], r.as_slice());
    assert_eq!(&raw[66..], s.as_slice());
}

#[test]
fn rejects_structural_damage() {
    let good = der(&[1], &[2]);
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty", vec![]),
        ("not a sequence", [&[0x31][..], &good[1..]].concat()),
        ("trailing bytes", [good.clone(), vec![0]].concat()),
        ("truncated", good[..good.len() - 1].to_vec()),
        (
            "indefinite length",
            vec![0x30, 0x80, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02, 0, 0],
        ),
        (
            "two-byte length",
            vec![0x30, 0x82, 0x00, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02],
        ),
        (
            "r not an integer",
            vec![0x30, 0x06, 0x04, 0x01, 0x01, 0x02, 0x01, 0x02],
        ),
        ("only r", vec![0x30, 0x03, 0x02, 0x01, 0x01]),
        (
            "extra element",
            vec![
                0x30, 0x09, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02, 0x02, 0x01, 0x03,
            ],
        ),
        (
            "bytes inside sequence after s",
            vec![0x30, 0x07, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02, 0xff],
        ),
        ("negative r", der(&[0x80], &[2])),
        ("negative s", der(&[1], &[0xff])),
        ("empty r", der(&[], &[2])),
        ("empty s", der(&[1], &[])),
        ("zero r", der(&[0x00], &[2])),
        ("zero s", der(&[1], &[0x00, 0x00])),
    ];
    for (why, bytes) in cases {
        assert_eq!(
            der_to_raw(&bytes, Curve::P256),
            Err(EcdsaEncodingError::Malformed),
            "{why}"
        );
    }
}

#[test]
fn raw_to_der_is_minimal() {
    let raw = raw_of(Curve::P256, &[0x01], &[0x80]);
    assert_eq!(
        raw_to_der(&raw, Curve::P256).unwrap(),
        vec![0x30, 0x07, 0x02, 0x01, 0x01, 0x02, 0x02, 0x00, 0x80]
    );
}

#[test]
fn raw_to_der_keeps_one_byte_for_zero() {
    let raw = vec![0; Curve::P256.signature_len()];
    assert_eq!(
        raw_to_der(&raw, Curve::P256).unwrap(),
        vec![0x30, 0x06, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00]
    );
}

#[test]
fn raw_to_der_uses_the_long_form_past_127_bytes() {
    let raw = vec![0xff; Curve::P521.signature_len()];
    let encoded = raw_to_der(&raw, Curve::P521).unwrap();
    // Each integer: 02 42 00 <66 bytes> = 69 bytes; two of them = 138 = 0x8a.
    assert_eq!(&encoded[..3], &[0x30, 0x81, 0x8a]);
    assert_eq!(encoded.len(), 3 + 138);
}

#[test]
fn raw_to_der_checks_the_length() {
    for curve in ALL {
        for actual in [0, 1, curve.signature_len() - 1, curve.signature_len() + 1] {
            assert_eq!(
                raw_to_der(&vec![1; actual], curve),
                Err(EcdsaEncodingError::WrongLength {
                    expected: curve.signature_len(),
                    actual
                })
            );
        }
    }
}

#[test]
fn round_trips_raw_signatures() {
    for curve in ALL {
        let n = curve.field_len();
        let samples: Vec<Vec<u8>> = vec![
            (0..2 * n).map(|i| (i % 251) as u8 | 1).collect(),
            [vec![0x80; n], vec![0x01; n]].concat(),
            [vec![0x00; n - 1], vec![0x01], vec![0xff; n]].concat(),
            [vec![0x7f; n], vec![0x00; n - 1], vec![0x01]].concat(),
        ];
        for raw in samples {
            let encoded = raw_to_der(&raw, curve).unwrap();
            assert_eq!(der_to_raw(&encoded, curve).unwrap(), raw);
        }
    }
}

#[test]
fn error_messages_are_stable() {
    assert_eq!(
        EcdsaEncodingError::Malformed.to_string(),
        "malformed ECDSA signature"
    );
    assert_eq!(
        EcdsaEncodingError::WrongLength {
            expected: 64,
            actual: 3
        }
        .to_string(),
        "raw ECDSA signature must be 64 bytes, got 3"
    );
}

#[test]
fn arbitrary_bytes_never_panic() {
    // Deterministic pseudo-random bytes: cheap stand-in for a fuzzer.
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..2000 {
        let len = (next() % 160) as usize;
        let mut bytes: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        if let Some(first) = bytes.first_mut() {
            *first = 0x30;
        }
        for curve in ALL {
            let _ = der_to_raw(&bytes, curve);
            let _ = raw_to_der(&bytes, curve);
        }
    }
}
