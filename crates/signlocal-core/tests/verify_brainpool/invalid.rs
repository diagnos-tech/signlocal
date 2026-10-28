//! Signatures that must not verify.

use super::*;

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
