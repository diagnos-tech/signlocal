//! SPEC §4.1 and §7: `verify` on Brainpool keys. The signatures were made by
//! OpenSSL (`vectors/brainpool-*.txt`); the crate under test never signs.

mod common;

use common::{
    BRAINPOOL_R1_KEYS, BRAINPOOL_TWISTED, brainpool_ecdsa_vectors, brainpool_signature_vectors,
    cert, fixture_digest, signature,
};
use websign_core::ecdsa::der_to_raw;
use websign_core::{
    CertInfo, Curve, DigestLengthError, HashAlgorithm, PublicKeyKind, SignatureAlgorithm,
    VerifyError, verify,
};

use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

const VERIFIABLE: [&str; 2] = ["brainpoolP256r1", "brainpoolP384r1"];

fn verify_fixture(
    key: &str,
    hash: HashAlgorithm,
    algorithm: SignatureAlgorithm,
    signature: &[u8],
) -> Result<(), VerifyError> {
    verify(
        &cert(key),
        hash,
        algorithm,
        &fixture_digest(hash),
        signature,
    )
}

fn key_for(curve: Curve) -> &'static str {
    BRAINPOOL_R1_KEYS
        .iter()
        .find(|(_, c)| *c == curve)
        .map(|(name, _)| *name)
        .expect("Brainpool fixture key")
}

/// Copy of `cert` whose EC point has its last byte changed, so the point is
/// off the curve (with overwhelming probability, and certainly not the same).
fn with_corrupted_point(cert: &[u8], point_len: usize) -> Vec<u8> {
    // BIT STRING header + unused-bits byte + uncompressed-point marker.
    let marker = [0x03, (point_len + 1) as u8, 0x00, 0x04];
    let at = cert
        .windows(marker.len())
        .position(|w| w == marker)
        .expect("EC point in the fixture");
    let last = at + 3 + point_len - 1;
    let mut out = cert.to_vec();
    out[last] ^= 0x01;
    out
}

// --- valid signatures --------------------------------------------------------------------------

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

fn signature_of(key: &str, hash: HashAlgorithm) -> Vec<u8> {
    brainpool_signature_vectors()
        .into_iter()
        .find(|v| v.key == key && v.hash == hash)
        .unwrap_or_else(|| panic!("no signature for {key} {hash}"))
        .signature
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
    // SPEC: CMS/PAdES do not require low s; the fixtures include high-s cases.
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

// --- invalid signatures ------------------------------------------------------------------------

#[test]
fn rejects_a_flipped_bit_anywhere_in_the_signature() {
    for key in VERIFIABLE {
        let sig = signature_of(key, HashAlgorithm::Sha256);
        for index in (0..sig.len()).step_by(7) {
            let mut bad = sig.clone();
            bad[index] ^= 0x10;
            assert_eq!(
                verify_fixture(key, HashAlgorithm::Sha256, Ecdsa, &bad),
                Err(VerifyError::InvalidSignature),
                "{key} byte {index}"
            );
        }
    }
}

#[test]
fn rejects_a_different_digest() {
    for key in VERIFIABLE {
        let sig = signature_of(key, HashAlgorithm::Sha256);
        let mut digest = fixture_digest(HashAlgorithm::Sha256);
        digest[0] ^= 1;
        assert_eq!(
            verify(&cert(key), HashAlgorithm::Sha256, Ecdsa, &digest, &sig),
            Err(VerifyError::InvalidSignature),
            "{key}"
        );
    }
}

#[test]
fn rejects_zero_and_out_of_range_r_and_s() {
    for key in VERIFIABLE {
        let good = signature_of(key, HashAlgorithm::Sha256);
        let len = good.len();
        let half = len / 2;
        // 2^bits - 1 is larger than any Brainpool group order of this size.
        let with_half = |range: std::ops::Range<usize>, value: u8| {
            let mut sig = good.clone();
            sig[range].fill(value);
            sig
        };
        let cases = [
            ("all zero", vec![0; len]),
            ("all ff", vec![0xFF; len]),
            ("r = 0", with_half(0..half, 0)),
            ("s = 0", with_half(half..len, 0)),
            ("r >= n", with_half(0..half, 0xFF)),
            ("s >= n", with_half(half..len, 0xFF)),
        ];
        for (label, sig) in cases {
            assert_eq!(
                verify_fixture(key, HashAlgorithm::Sha256, Ecdsa, &sig),
                Err(VerifyError::InvalidSignature),
                "{key} {label}"
            );
        }
    }
}

#[test]
fn rejects_signatures_of_the_wrong_size_and_the_der_form() {
    for key in VERIFIABLE {
        let sig = signature_of(key, HashAlgorithm::Sha256);
        let mut too_long = sig.clone();
        too_long.push(0);
        let bad_sizes = [
            Vec::new(),
            sig[..sig.len() - 1].to_vec(),
            too_long,
            sig[..sig.len() / 2].to_vec(),
        ];
        for bad in bad_sizes {
            assert_eq!(
                verify_fixture(key, HashAlgorithm::Sha256, Ecdsa, &bad),
                Err(VerifyError::InvalidSignature),
                "{key} len {}",
                bad.len()
            );
        }
    }
    for v in brainpool_ecdsa_vectors()
        .into_iter()
        .filter(|v| v.curve.has_verifier())
    {
        assert_eq!(
            verify_fixture(key_for(v.curve), v.hash, Ecdsa, &v.der),
            Err(VerifyError::InvalidSignature),
            "{:?} {}",
            v.curve,
            v.shape
        );
    }
}

#[test]
fn a_signature_of_one_curve_does_not_verify_on_another_of_the_same_size() {
    let brainpool = signature_of("brainpoolP256r1", HashAlgorithm::Sha256);
    assert_eq!(
        verify_fixture("p256", HashAlgorithm::Sha256, Ecdsa, &brainpool),
        Err(VerifyError::InvalidSignature)
    );
    let nist = signature("p256", "ecdsa", HashAlgorithm::Sha256);
    assert_eq!(
        verify_fixture("brainpoolP256r1", HashAlgorithm::Sha256, Ecdsa, &nist),
        Err(VerifyError::InvalidSignature)
    );
    let nist384 = signature("p384", "ecdsa", HashAlgorithm::Sha384);
    assert_eq!(
        verify_fixture("brainpoolP384r1", HashAlgorithm::Sha384, Ecdsa, &nist384),
        Err(VerifyError::InvalidSignature)
    );
}

// --- check order -------------------------------------------------------------------------------

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
