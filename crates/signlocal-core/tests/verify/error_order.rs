//! Error order.

use super::*;

#[test]
fn reports_a_bad_certificate_before_anything_else() {
    // Wrong digest size, impossible algorithm/key pairing and an empty signature
    // are all present too; the certificate comes first.
    let bad_certs: Vec<(&str, Vec<u8>)> = vec![
        ("empty", vec![]),
        ("garbage", vec![1, 2, 3]),
        ("truncated", cert("p256")[..100].to_vec()),
        ("trailing byte", [cert("p256"), vec![0]].concat()),
        ("malformed extension", cert("bad-ext-key-usage")),
    ];
    for (label, der) in bad_certs {
        let result = verify(&der, Sha256, Ecdsa, &[0; 5], &[]);
        let expected = CertInfo::from_der(&der).unwrap_err();
        assert_eq!(result, Err(VerifyError::Certificate(expected)), "{label}");
    }
}

#[test]
fn reports_a_wrong_digest_length_before_key_and_signature_problems() {
    let cases = [
        ("p256", Ecdsa),
        ("rsa2048", Ecdsa),
        ("p256", RsaPss),
        ("ed25519", RsaPkcs1v15),
        ("secp256k1", Ecdsa),
    ];
    for (key, algorithm) in cases {
        for hash in HashAlgorithm::ALL {
            for len in [0, 20, hash.digest_len() - 1, hash.digest_len() + 1] {
                let digest = vec![0x42; len];
                let result = verify(&cert(key), hash, algorithm, &digest, &[]);
                let expected = DigestLengthError {
                    algorithm: hash,
                    expected: hash.digest_len(),
                    actual: len,
                };
                assert_eq!(
                    result,
                    Err(VerifyError::Digest(expected)),
                    "{key} {algorithm} {hash} len {len}"
                );
            }
        }
    }
}

#[test]
fn reports_an_unsupported_key_before_algorithm_and_signature_problems() {
    for key in UNSUPPORTED_KEYS {
        for algorithm in SignatureAlgorithm::ALL {
            for hash in HashAlgorithm::ALL {
                let result = verify_fixture(key, hash, algorithm, &[]);
                assert_eq!(
                    result,
                    Err(VerifyError::UnsupportedKey),
                    "{key} {algorithm} {hash}"
                );
            }
        }
    }
}

#[test]
fn reports_a_key_mismatch_before_looking_at_the_signature() {
    let junk_signatures: [&[u8]; 3] = [&[], &[1, 2, 3], &[0; 300]];
    for key in ["rsa2048", "rsa3072", "rsapss2048"] {
        for junk in junk_signatures {
            let result = verify_fixture(key, Sha256, Ecdsa, junk);
            assert_eq!(result, Err(VerifyError::KeyMismatch(Ecdsa)), "{key}");
        }
    }
    for key in ["p256", "p384", "p521"] {
        for junk in junk_signatures {
            for algorithm in [RsaPkcs1v15, RsaPss] {
                let result = verify_fixture(key, Sha256, algorithm, junk);
                assert_eq!(
                    result,
                    Err(VerifyError::KeyMismatch(algorithm)),
                    "{key} {algorithm}"
                );
            }
        }
    }
}

#[test]
fn key_mismatch_is_reported_even_for_a_real_signature_of_another_kind() {
    let ecdsa_sig = signature("p256", "ecdsa", Sha256);
    assert_eq!(
        verify_fixture("rsa2048", Sha256, Ecdsa, &ecdsa_sig),
        Err(VerifyError::KeyMismatch(Ecdsa))
    );
    let rsa_sig = signature("rsa2048", "pkcs1", Sha256);
    assert_eq!(
        verify_fixture("p256", Sha256, RsaPkcs1v15, &rsa_sig),
        Err(VerifyError::KeyMismatch(RsaPkcs1v15))
    );
    assert_eq!(
        verify_fixture("p256", Sha256, RsaPss, &rsa_sig),
        Err(VerifyError::KeyMismatch(RsaPss))
    );
}
