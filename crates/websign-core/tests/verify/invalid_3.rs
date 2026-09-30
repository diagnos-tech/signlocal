//! Invalid signatures (part 3 of 3).

use super::*;

#[test]
fn rejects_a_signature_from_a_key_of_another_size() {
    let rsa_2048 = signature("rsa2048", "pkcs1", Sha256);
    assert_eq!(
        verify_fixture("rsa3072", Sha256, RsaPkcs1v15, &rsa_2048),
        Err(VerifyError::InvalidSignature)
    );
    let rsa_3072 = signature("rsa3072", "pkcs1", Sha256);
    assert_eq!(
        verify_fixture("rsa2048", Sha256, RsaPkcs1v15, &rsa_3072),
        Err(VerifyError::InvalidSignature)
    );
    let p256 = signature("p256", "ecdsa", Sha256);
    assert_eq!(
        verify_fixture("p384", Sha256, Ecdsa, &p256),
        Err(VerifyError::InvalidSignature)
    );
    let p384 = signature("p384", "ecdsa", Sha384);
    assert_eq!(
        verify_fixture("p256", Sha384, Ecdsa, &p384),
        Err(VerifyError::InvalidSignature)
    );
    assert_eq!(
        verify_fixture("p521", Sha384, Ecdsa, &p384),
        Err(VerifyError::InvalidSignature)
    );
}

#[test]
fn rejects_pkcs1_signatures_verified_as_pss_and_the_other_way_round() {
    for key in ["rsa2048", "rsa3072", "rsa4096", "rsa2047"] {
        for hash in HashAlgorithm::ALL {
            let pkcs1 = signature(key, "pkcs1", hash);
            let pss = signature(key, "pss", hash);
            assert_eq!(
                verify_fixture(key, hash, RsaPss, &pkcs1),
                Err(VerifyError::InvalidSignature),
                "{key} {hash} pkcs1 as pss"
            );
            assert_eq!(
                verify_fixture(key, hash, RsaPkcs1v15, &pss),
                Err(VerifyError::InvalidSignature),
                "{key} {hash} pss as pkcs1"
            );
        }
    }
}

#[test]
fn rejects_pss_signatures_that_do_not_use_the_agreed_parameters() {
    // MGF1 over the same hash and a salt as long as the digest. The
    // manifest has signatures made with salt 0, salt 20 and MGF1-SHA1; a
    // verifier that auto-detects the salt or MGF1 hash would accept them.
    for kind in ["pss-salt-0", "pss-salt-20", "pss-mgf1-sha1"] {
        for hash in HashAlgorithm::ALL {
            let sig = signature("rsa2048", kind, hash);
            assert_eq!(
                verify_fixture("rsa2048", hash, RsaPss, &sig),
                Err(VerifyError::InvalidSignature),
                "{kind} with {hash}"
            );
        }
    }
}

#[test]
fn signature_garbage_of_any_length_is_an_invalid_signature_not_a_panic() {
    let cases = [
        ("rsa2048", RsaPkcs1v15),
        ("rsa2048", RsaPss),
        ("p256", Ecdsa),
        ("p521", Ecdsa),
    ];
    for (key, algorithm) in cases {
        for len in [
            0, 1, 2, 31, 32, 33, 63, 64, 65, 131, 132, 133, 255, 256, 257, 1000,
        ] {
            let sig: Vec<u8> = (0..len).map(|i| (i * 31 + 7) as u8).collect();
            assert_eq!(
                verify_fixture(key, Sha256, algorithm, &sig),
                Err(VerifyError::InvalidSignature),
                "{key} {algorithm} len {len}"
            );
        }
    }
}
