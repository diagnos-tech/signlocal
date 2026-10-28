//! DER and raw encodings on the Brainpool field sizes.

use super::*;

fn raw_of(curve: Curve, r: &[u8], s: &[u8]) -> Vec<u8> {
    let mut out = left_pad(r, curve.field_len());
    out.extend(left_pad(s, curve.field_len()));
    out
}

#[test]
fn pads_small_integers_to_the_brainpool_field_size() {
    let der = der_ecdsa(&[1], &[2]);
    for curve in BRAINPOOL {
        assert_eq!(
            der_to_raw(&der, curve),
            Ok(raw_of(curve, &[1], &[2])),
            "{curve:?}"
        );
    }
}

#[test]
fn keeps_full_size_integers_on_brainpool_curves() {
    for curve in BRAINPOOL {
        let (r, s) = (
            counting_bytes(1, curve.field_len()),
            counting_bytes(0x41, curve.field_len()),
        );
        let expected = [r.clone(), s.clone()].concat();
        assert_eq!(
            der_to_raw(&der_ecdsa(&r, &s), curve),
            Ok(expected),
            "{curve:?}"
        );
    }
}

#[test]
fn rejects_an_integer_one_byte_too_large_for_the_brainpool_field() {
    for curve in BRAINPOOL {
        let too_large = counting_bytes(1, curve.field_len() + 1);
        let fits = counting_bytes(1, curve.field_len());
        assert_eq!(
            der_to_raw(&der_ecdsa(&too_large, &fits), curve),
            Err(EcdsaEncodingError::IntegerTooLarge),
            "{curve:?} r"
        );
        assert_eq!(
            der_to_raw(&der_ecdsa(&fits, &too_large), curve),
            Err(EcdsaEncodingError::IntegerTooLarge),
            "{curve:?} s"
        );
    }
}

#[test]
fn a_p256_sized_signature_is_not_a_brainpool_p384_signature() {
    let wrong = vec![7u8; 64];
    assert_eq!(
        raw_to_der(&wrong, BrainpoolP384r1),
        Err(EcdsaEncodingError::WrongLength {
            expected: 96,
            actual: 64
        })
    );
    assert_eq!(
        raw_to_der(&wrong, BrainpoolP512r1),
        Err(EcdsaEncodingError::WrongLength {
            expected: 128,
            actual: 64
        })
    );
    assert!(raw_to_der(&wrong, BrainpoolP256r1).is_ok());
}

#[test]
fn round_trips_random_values_on_brainpool_curves() {
    let mut rng = Rng::new(0x00B2_A177);
    for curve in BRAINPOOL {
        for _ in 0..200 {
            let mut raw = rng.bytes(curve.signature_len());
            let n = curve.field_len();
            // Non-zero r and s, and sometimes short halves.
            raw[n - 1] |= 1;
            raw[2 * n - 1] |= 1;
            if rng.below(4) == 0 {
                raw[0] = 0;
            }
            if rng.below(4) == 0 {
                raw[n] = 0;
            }
            let der = raw_to_der(&raw, curve).expect("right length");
            assert_eq!(der_to_raw(&der, curve), Ok(raw), "{curve:?}");
        }
    }
}

#[test]
fn brainpool_p512r1_der_uses_the_long_length_form_when_needed() {
    // Two 64-byte halves with the high bit set: 2 * (2 + 1 + 64) = 134 > 127.
    let raw = vec![0xC3; 128];
    let der = raw_to_der(&raw, BrainpoolP512r1).expect("right length");
    assert_eq!(der[..2], [0x30, 0x81]);
    assert_eq!(der[2] as usize, der.len() - 3);
    assert!(parse_strict_ecdsa_der(&der).is_some());
    assert_eq!(der_to_raw(&der, BrainpoolP512r1), Ok(raw));
}
