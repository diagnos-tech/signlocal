//! Valid signatures: every hash × algorithm × key.

use super::*;

macro_rules! accepts_every_hash {
    ($test:ident, $key:literal, $algorithm:expr, $kind:literal) => {
        #[test]
        fn $test() {
            for hash in HashAlgorithm::ALL {
                let sig = signature($key, $kind, hash);
                assert_eq!(
                    verify_fixture($key, hash, $algorithm, &sig),
                    Ok(()),
                    "{} {} with {hash}",
                    $key,
                    $kind
                );
            }
        }
    };
}

accepts_every_hash!(
    accepts_rsa_2048_pkcs1v15_with_every_hash,
    "rsa2048",
    RsaPkcs1v15,
    "pkcs1"
);
accepts_every_hash!(
    accepts_rsa_2048_pss_with_every_hash,
    "rsa2048",
    RsaPss,
    "pss"
);
accepts_every_hash!(
    accepts_rsa_3072_pkcs1v15_with_every_hash,
    "rsa3072",
    RsaPkcs1v15,
    "pkcs1"
);
accepts_every_hash!(
    accepts_rsa_3072_pss_with_every_hash,
    "rsa3072",
    RsaPss,
    "pss"
);
accepts_every_hash!(
    accepts_rsa_4096_pkcs1v15_with_every_hash,
    "rsa4096",
    RsaPkcs1v15,
    "pkcs1"
);
accepts_every_hash!(
    accepts_rsa_4096_pss_with_every_hash,
    "rsa4096",
    RsaPss,
    "pss"
);
accepts_every_hash!(
    accepts_rsa_2047_pkcs1v15_with_every_hash,
    "rsa2047",
    RsaPkcs1v15,
    "pkcs1"
);
accepts_every_hash!(
    accepts_rsa_2047_pss_with_every_hash,
    "rsa2047",
    RsaPss,
    "pss"
);
accepts_every_hash!(
    accepts_rsa_pss_typed_key_with_every_hash,
    "rsapss2048",
    RsaPss,
    "pss"
);
accepts_every_hash!(accepts_ecdsa_p256_with_every_hash, "p256", Ecdsa, "ecdsa");
accepts_every_hash!(accepts_ecdsa_p384_with_every_hash, "p384", Ecdsa, "ecdsa");
accepts_every_hash!(accepts_ecdsa_p521_with_every_hash, "p521", Ecdsa, "ecdsa");

#[test]
fn accepts_every_valid_vector_in_the_manifest() {
    let vectors = valid_vectors();
    // 5 RSA keys x 3 hashes x 2 paddings + 3 RSA-PSS-key + 4 EC keys x 3 hashes.
    assert_eq!(vectors.len(), 45, "signature manifest changed shape");
    for vector in vectors {
        assert_eq!(
            check(&vector),
            Ok(()),
            "{} {} {}",
            vector.key,
            vector.algorithm,
            vector.hash
        );
    }
}

#[test]
fn accepts_ecdsa_when_the_hash_is_longer_or_shorter_than_the_curve() {
    // FIPS 186-5: the digest is truncated or used as is. P-256+SHA-384,
    // P-256+SHA-512, P-384+SHA-256, P-521+SHA-256 are all valid pairings.
    for (key, hash) in [
        ("p256", Sha384),
        ("p256", Sha512),
        ("p384", Sha256),
        ("p384", Sha512),
        ("p521", Sha256),
        ("p521", Sha384),
    ] {
        let sig = signature(key, "ecdsa", hash);
        assert_eq!(
            verify_fixture(key, hash, Ecdsa, &sig),
            Ok(()),
            "{key}+{hash}"
        );
    }
}

#[test]
fn accepts_the_raw_form_converted_from_openssl_der() {
    for vector in ecdsa_vectors() {
        let key = match vector.curve {
            Curve::P256 => "p256",
            Curve::P384 => "p384",
            Curve::P521 => "p521",
            other => panic!("no fixture key for {other:?}"),
        };
        let raw = der_to_raw(&vector.der, vector.curve).expect("valid DER");
        assert_eq!(
            verify_fixture(key, vector.hash, Ecdsa, &raw),
            Ok(()),
            "{key} {}",
            vector.shape
        );
    }
}

#[test]
fn checks_certificate_bytes_only_for_the_key_not_for_validity_or_usage() {
    // SPEC lists five checks and none is a validity period or KeyUsage check:
    // an expired certificate and one limited to keyEncipherment share the
    // p256 key, so the p256 signature verifies against them.
    let sig = signature("p256", "ecdsa", Sha256);
    for name in ["expired", "ku-key-encipherment", "ca-pathlen0", "icp-pf-a3"] {
        assert_eq!(verify_fixture(name, Sha256, Ecdsa, &sig), Ok(()), "{name}");
    }
}
