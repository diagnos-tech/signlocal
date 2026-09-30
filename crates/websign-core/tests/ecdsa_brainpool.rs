//! SPEC §4 and §4.1: `Curve` with the Brainpool curves, OID mapping, and the
//! signature encodings on curves with 32, 48 and 64 byte fields.

mod common;

use std::collections::HashSet;

use common::{
    BRAINPOOL_R1_KEYS, BRAINPOOL_TWISTED, Rng, brainpool_ecdsa_vectors, counting_bytes, der_ecdsa,
    left_pad, parse_strict_ecdsa_der,
};
use websign_core::ecdsa::{Curve, EcdsaEncodingError, der_to_raw, raw_to_der};

use Curve::{BrainpoolP256r1, BrainpoolP384r1, BrainpoolP512r1, P256, P384, P521};

const BRAINPOOL: [Curve; 3] = [BrainpoolP256r1, BrainpoolP384r1, BrainpoolP512r1];
const ALL: [Curve; 6] = [
    P256,
    P384,
    P521,
    BrainpoolP256r1,
    BrainpoolP384r1,
    BrainpoolP512r1,
];

// --- Curve ---------------------------------------------------------------------------------

#[test]
fn reports_field_length_in_bytes() {
    let expected = [32, 48, 66, 32, 48, 64];
    for (curve, len) in ALL.into_iter().zip(expected) {
        assert_eq!(curve.field_len(), len, "{curve:?}");
    }
}

#[test]
fn reports_raw_signature_length_as_twice_the_field() {
    for curve in ALL {
        assert_eq!(curve.signature_len(), 2 * curve.field_len(), "{curve:?}");
    }
    assert_eq!(BrainpoolP256r1.signature_len(), 64);
    assert_eq!(BrainpoolP384r1.signature_len(), 96);
    assert_eq!(BrainpoolP512r1.signature_len(), 128);
}

#[test]
fn reports_curve_names() {
    assert_eq!(BrainpoolP256r1.name(), "brainpoolP256r1");
    assert_eq!(BrainpoolP384r1.name(), "brainpoolP384r1");
    assert_eq!(BrainpoolP512r1.name(), "brainpoolP512r1");
    let names: HashSet<&str> = ALL.iter().map(|c| c.name()).collect();
    assert_eq!(names.len(), 6);
}

#[test]
fn only_brainpool_p512r1_lacks_a_verifier() {
    for curve in ALL {
        assert_eq!(curve.has_verifier(), curve != BrainpoolP512r1, "{curve:?}");
    }
}

// --- from_oid ------------------------------------------------------------------------------

#[test]
fn maps_the_brainpool_r1_oids() {
    assert_eq!(
        Curve::from_oid("1.3.36.3.3.2.8.1.1.7"),
        Some(BrainpoolP256r1)
    );
    assert_eq!(
        Curve::from_oid("1.3.36.3.3.2.8.1.1.11"),
        Some(BrainpoolP384r1)
    );
    assert_eq!(
        Curve::from_oid("1.3.36.3.3.2.8.1.1.13"),
        Some(BrainpoolP512r1)
    );
}

#[test]
fn keeps_the_nist_oids_working() {
    assert_eq!(Curve::from_oid("1.2.840.10045.3.1.7"), Some(P256));
    assert_eq!(Curve::from_oid("1.3.132.0.34"), Some(P384));
    assert_eq!(Curve::from_oid("1.3.132.0.35"), Some(P521));
}

#[test]
fn leaves_the_twisted_brainpool_oids_unknown() {
    for (_, oid) in BRAINPOOL_TWISTED {
        assert_eq!(Curve::from_oid(oid), None, "{oid}");
    }
}

#[test]
fn rejects_brainpool_look_alike_oids() {
    let unknown = [
        "1.3.36.3.3.2.8.1.1.5",  // brainpoolP224r1
        "1.3.36.3.3.2.8.1.1.6",  // brainpoolP224t1
        "1.3.36.3.3.2.8.1.1.9",  // P320r1
        "1.3.36.3.3.2.8.1.1.10", // P320t1
        "1.3.36.3.3.2.8.1.1.15",
        "1.3.36.3.3.2.8.1.1.70",
        "1.3.36.3.3.2.8.1.1.7.1",
        "1.3.36.3.3.2.8.1.1.7 ",
        " 1.3.36.3.3.2.8.1.1.11",
        "1.3.36.3.3.2.8.1.1",
        "1.3.36.3.3.2.8.1.1.113",
        "1.3.36.3.3.2.8.1.1.07",
        "brainpoolP256r1",
        "brainpoolP384r1",
    ];
    for oid in unknown {
        assert_eq!(Curve::from_oid(oid), None, "{oid:?}");
    }
}

#[test]
fn the_six_curve_oids_are_distinct() {
    let oids = [
        "1.2.840.10045.3.1.7",
        "1.3.132.0.34",
        "1.3.132.0.35",
        "1.3.36.3.3.2.8.1.1.7",
        "1.3.36.3.3.2.8.1.1.11",
        "1.3.36.3.3.2.8.1.1.13",
    ];
    let curves: HashSet<Curve> = oids
        .iter()
        .map(|oid| Curve::from_oid(oid).expect("known curve"))
        .collect();
    assert_eq!(curves.len(), 6);
}

// --- encodings on the new field sizes --------------------------------------------------------

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

// --- OpenSSL vectors ---------------------------------------------------------------------------

#[test]
fn openssl_fixtures_cover_every_shape_on_every_brainpool_curve() {
    let vectors = brainpool_ecdsa_vectors();
    for curve in BRAINPOOL {
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
fn converts_openssl_der_to_the_raw_form_openssl_readback_gives() {
    for v in brainpool_ecdsa_vectors() {
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
    for v in brainpool_ecdsa_vectors() {
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
fn openssl_brainpool_vectors_use_the_field_sizes_of_the_curve() {
    let pairs: Vec<_> = BRAINPOOL_R1_KEYS.iter().map(|(_, c)| *c).collect();
    for v in brainpool_ecdsa_vectors() {
        assert!(pairs.contains(&v.curve));
        assert_eq!(v.raw.len(), v.curve.signature_len(), "{:?}", v.curve);
    }
}

#[test]
fn a_brainpool_p384_signature_does_not_fit_brainpool_p256() {
    for v in brainpool_ecdsa_vectors()
        .into_iter()
        .filter(|v| v.curve == BrainpoolP384r1)
    {
        assert_eq!(
            der_to_raw(&v.der, BrainpoolP256r1),
            Err(EcdsaEncodingError::IntegerTooLarge),
            "{}",
            v.shape
        );
    }
}
