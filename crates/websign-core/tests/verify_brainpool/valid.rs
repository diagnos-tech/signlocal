//! Signatures that verify.

use super::*;

#[test]
fn accepts_brainpool_p256r1_and_p384r1_signatures_with_every_hash() {
    for key in VERIFIABLE {
        for hash in HashAlgorithm::ALL {
            let sig = signature_of(key, hash);
            assert_eq!(
                verify_fixture(key, hash, Ecdsa, &sig),
                Ok(()),
                "{key} {hash}"
            );
        }
    }
}

#[test]
fn accepts_the_raw_form_converted_from_openssl_der_in_every_shape() {
    for v in brainpool_ecdsa_vectors() {
        if !v.curve.has_verifier() {
            continue;
        }
        let raw = der_to_raw(&v.der, v.curve).expect("valid DER");
        assert_eq!(
            verify_fixture(key_for(v.curve), v.hash, Ecdsa, &raw),
            Ok(()),
            "{:?} {}",
            v.curve,
            v.shape
        );
    }
}

#[test]
fn accepts_high_s_signatures() {
    // §7: CMS/PAdES do not require low s; the fixtures include high-s cases.
    let shapes: Vec<_> = brainpool_ecdsa_vectors()
        .into_iter()
        .filter(|v| v.curve.has_verifier() && matches!(v.shape.as_str(), "high-s" | "high-both"))
        .collect();
    assert!(!shapes.is_empty());
    for v in shapes {
        assert_eq!(
            verify_fixture(key_for(v.curve), v.hash, Ecdsa, &v.raw),
            Ok(()),
            "{:?} {}",
            v.curve,
            v.shape
        );
    }
}

#[test]
fn a_signature_is_valid_for_the_hash_it_was_made_with_and_no_other() {
    for key in VERIFIABLE {
        for signed_with in HashAlgorithm::ALL {
            let sig = signature_of(key, signed_with);
            for checked_with in HashAlgorithm::ALL {
                let result = verify_fixture(key, checked_with, Ecdsa, &sig);
                if checked_with == signed_with {
                    assert_eq!(result, Ok(()), "{key} {signed_with}");
                } else {
                    assert_eq!(
                        result,
                        Err(VerifyError::InvalidSignature),
                        "{key} signed with {signed_with}, checked with {checked_with}"
                    );
                }
            }
        }
    }
}
