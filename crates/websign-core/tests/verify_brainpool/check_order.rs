//! Which error wins, and unusable keys.

use super::*;

#[test]
fn rsa_algorithms_do_not_match_brainpool_keys() {
    for (key, _) in BRAINPOOL_R1_KEYS {
        for algorithm in [RsaPkcs1v15, RsaPss] {
            assert_eq!(
                verify_fixture(key, HashAlgorithm::Sha256, algorithm, &[0; 64]),
                Err(VerifyError::KeyMismatch(algorithm)),
                "{key} {algorithm}"
            );
        }
    }
}

#[test]
fn brainpool_p512r1_has_no_verifier_so_even_a_valid_signature_is_unsupported() {
    for hash in HashAlgorithm::ALL {
        let sig = signature_of("brainpoolP512r1", hash);
        assert_eq!(sig.len(), 128);
        assert_eq!(
            verify_fixture("brainpoolP512r1", hash, Ecdsa, &sig),
            Err(VerifyError::UnsupportedKey),
            "{hash}"
        );
    }
}

#[test]
fn brainpool_p512r1_is_unsupported_before_the_signature_is_looked_at() {
    for junk in [&[][..], &[1, 2, 3][..], &[0; 128][..], &[0xFF; 200][..]] {
        assert_eq!(
            verify_fixture("brainpoolP512r1", HashAlgorithm::Sha512, Ecdsa, junk),
            Err(VerifyError::UnsupportedKey),
            "len {}",
            junk.len()
        );
    }
}

#[test]
fn brainpool_p512r1_still_reports_mismatch_and_digest_errors_first() {
    assert_eq!(
        verify_fixture("brainpoolP512r1", HashAlgorithm::Sha512, RsaPss, &[0; 128]),
        Err(VerifyError::KeyMismatch(RsaPss))
    );
    let result = verify(
        &cert("brainpoolP512r1"),
        HashAlgorithm::Sha512,
        Ecdsa,
        &[0; 10],
        &[0; 128],
    );
    assert_eq!(
        result,
        Err(VerifyError::Digest(DigestLengthError {
            algorithm: HashAlgorithm::Sha512,
            expected: 64,
            actual: 10
        }))
    );
}

#[test]
fn reports_digest_length_errors_on_brainpool_keys() {
    for key in VERIFIABLE {
        let sig = signature_of(key, HashAlgorithm::Sha256);
        for len in [0, 31, 33, 48] {
            let result = verify(
                &cert(key),
                HashAlgorithm::Sha256,
                Ecdsa,
                &vec![0; len],
                &sig,
            );
            assert_eq!(
                result,
                Err(VerifyError::Digest(DigestLengthError {
                    algorithm: HashAlgorithm::Sha256,
                    expected: 32,
                    actual: len
                })),
                "{key} len {len}"
            );
        }
    }
}

#[test]
fn a_certificate_that_is_not_one_is_a_certificate_error() {
    let result = verify(&[1, 2, 3], HashAlgorithm::Sha256, Ecdsa, &[0; 32], &[0; 64]);
    assert!(matches!(result, Err(VerifyError::Certificate(_))));
}

#[test]
fn a_point_off_the_curve_is_an_unusable_key() {
    for (key, point_len) in [("brainpoolP256r1", 65), ("brainpoolP384r1", 97)] {
        let sig = signature_of(key, HashAlgorithm::Sha256);
        let broken = with_corrupted_point(&cert(key), point_len);
        // The summary does not check the point (SPEC §6.2), verify does.
        assert!(matches!(
            CertInfo::from_der(&broken).map(|i| i.key),
            Ok(PublicKeyKind::Ec { .. })
        ));
        assert_eq!(
            verify(
                &broken,
                HashAlgorithm::Sha256,
                Ecdsa,
                &fixture_digest(HashAlgorithm::Sha256),
                &sig
            ),
            Err(VerifyError::UnsupportedKey),
            "{key}"
        );
    }
}

#[test]
fn twisted_brainpool_keys_stay_unsupported() {
    for (key, _) in BRAINPOOL_TWISTED {
        for algorithm in SignatureAlgorithm::ALL {
            assert_eq!(
                verify_fixture(key, HashAlgorithm::Sha256, algorithm, &[0; 64]),
                Err(VerifyError::UnsupportedKey),
                "{key} {algorithm}"
            );
        }
    }
}
