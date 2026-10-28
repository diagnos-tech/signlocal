//! Round trip and OpenSSL vectors.

use super::*;

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
