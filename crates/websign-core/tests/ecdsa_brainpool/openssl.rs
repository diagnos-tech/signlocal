//! The OpenSSL Brainpool vectors.

use super::*;

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
