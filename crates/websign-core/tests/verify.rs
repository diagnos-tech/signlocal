//! SPEC §7: `verify`, `VerifyError`.
//!
//! Every valid signature here was made by OpenSSL over the digest of the
//! fixture message (`tests/fixtures/vectors/signatures.txt`); the crate under
//! test never signs.

mod common;

use common::{
    SignatureVector, cert, ecdsa_vectors, fixture_digest, hex_decode, left_pad, rsa_modulus,
    signature, signature_vectors,
};
use websign_core::ecdsa::der_to_raw;
use websign_core::{
    CertError, CertInfo, Curve, DigestLengthError, HashAlgorithm, SignatureAlgorithm, VerifyError,
    verify,
};

use HashAlgorithm::{Sha256, Sha384, Sha512};
use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

fn algorithm_of(manifest_name: &str) -> SignatureAlgorithm {
    match manifest_name {
        "pkcs1" => RsaPkcs1v15,
        "pss" => RsaPss,
        "ecdsa" => Ecdsa,
        other => panic!("not a valid-signature kind: {other}"),
    }
}

/// The signatures the SDK promises to be valid.
fn valid_vectors() -> Vec<SignatureVector> {
    signature_vectors()
        .into_iter()
        .filter(|v| matches!(v.algorithm.as_str(), "pkcs1" | "pss" | "ecdsa"))
        .collect()
}

fn check(vector: &SignatureVector) -> Result<(), VerifyError> {
    verify(
        &cert(&vector.key),
        vector.hash,
        algorithm_of(&vector.algorithm),
        &fixture_digest(vector.hash),
        &vector.signature,
    )
}

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

const UNSUPPORTED_KEYS: [&str; 5] = ["ed25519", "ed448", "secp256k1", "secp224r1", "dsa1024"];

// --- valid signatures: every hash x algorithm x key ------------------------------------------

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

// --- invalid signatures --------------------------------------------------------------------------

#[test]
fn rejects_a_digest_that_was_not_signed() {
    for vector in valid_vectors() {
        let digest = fixture_digest(vector.hash);
        // ECDSA uses only the leftmost field-size bytes of a longer digest
        // (FIPS 186-5), so flips beyond them are legitimately invisible.
        let significant = match vector.key.as_str() {
            "p256" | "p256b" => digest.len().min(32),
            "p384" => digest.len().min(48),
            _ => digest.len(),
        };
        for index in [0, significant / 2, significant - 1] {
            let mut tampered = digest.clone();
            tampered[index] ^= 0x01;
            let result = verify(
                &cert(&vector.key),
                vector.hash,
                algorithm_of(&vector.algorithm),
                &tampered,
                &vector.signature,
            );
            assert_eq!(
                result,
                Err(VerifyError::InvalidSignature),
                "{} {} byte {index}",
                vector.key,
                vector.algorithm
            );
        }
    }
}

#[test]
fn rejects_a_signature_with_one_flipped_bit() {
    for vector in valid_vectors() {
        let len = vector.signature.len();
        for index in [0, len / 2, len - 1] {
            for mask in [0x01, 0x80] {
                let mut tampered = vector.signature.clone();
                tampered[index] ^= mask;
                let result = verify(
                    &cert(&vector.key),
                    vector.hash,
                    algorithm_of(&vector.algorithm),
                    &fixture_digest(vector.hash),
                    &tampered,
                );
                assert_eq!(
                    result,
                    Err(VerifyError::InvalidSignature),
                    "{} {} {} byte {index} ^ {mask:#x}",
                    vector.key,
                    vector.algorithm,
                    vector.hash
                );
            }
        }
    }
}

#[test]
fn rejects_a_signature_made_over_another_hash_of_the_same_message() {
    for vector in valid_vectors() {
        for other in HashAlgorithm::ALL.into_iter().filter(|h| *h != vector.hash) {
            let result = verify(
                &cert(&vector.key),
                other,
                algorithm_of(&vector.algorithm),
                &fixture_digest(other),
                &vector.signature,
            );
            assert_eq!(
                result,
                Err(VerifyError::InvalidSignature),
                "{} {} {} as {other}",
                vector.key,
                vector.algorithm,
                vector.hash
            );
        }
    }
}

#[test]
fn rejects_a_signature_of_the_wrong_length() {
    for vector in valid_vectors() {
        let sig = &vector.signature;
        let mut with_zero_prefix = vec![0];
        with_zero_prefix.extend_from_slice(sig);
        let mut with_zero_suffix = sig.clone();
        with_zero_suffix.push(0);
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("empty", vec![]),
            ("one byte", vec![sig[0]]),
            ("one byte short (tail)", sig[..sig.len() - 1].to_vec()),
            ("one byte short (head)", sig[1..].to_vec()),
            ("one byte long (suffix)", with_zero_suffix),
            ("one byte long (prefix)", with_zero_prefix),
            ("doubled", [sig.as_slice(), sig.as_slice()].concat()),
        ];
        for (label, bad) in cases {
            let result = verify(
                &cert(&vector.key),
                vector.hash,
                algorithm_of(&vector.algorithm),
                &fixture_digest(vector.hash),
                &bad,
            );
            assert_eq!(
                result,
                Err(VerifyError::InvalidSignature),
                "{} {}: {label}",
                vector.key,
                vector.algorithm
            );
        }
    }
}

#[test]
fn rejects_ecdsa_in_der_form_because_the_sdk_promises_raw() {
    for vector in ecdsa_vectors() {
        let key = match vector.curve {
            Curve::P256 => "p256",
            Curve::P384 => "p384",
            Curve::P521 => "p521",
            other => panic!("no fixture key for {other:?}"),
        };
        assert_eq!(
            verify_fixture(key, vector.hash, Ecdsa, &vector.der),
            Err(VerifyError::InvalidSignature),
            "{key} {}",
            vector.shape
        );
    }
}

#[test]
fn rejects_degenerate_signature_blocks_of_the_right_size() {
    let sizes = [
        ("rsa2048", RsaPkcs1v15, 256),
        ("rsa2048", RsaPss, 256),
        ("rsa3072", RsaPkcs1v15, 384),
        ("p256", Ecdsa, 64),
        ("p384", Ecdsa, 96),
        ("p521", Ecdsa, 132),
    ];
    for (key, algorithm, len) in sizes {
        for (label, fill) in [("all zero", 0x00), ("all ff", 0xFF), ("all 01", 0x01)] {
            let result = verify_fixture(key, Sha256, algorithm, &vec![fill; len]);
            assert_eq!(
                result,
                Err(VerifyError::InvalidSignature),
                "{key} {algorithm}: {label}"
            );
        }
    }
}

/// `a + b` for big-endian numbers of equal length; the sum must fit.
fn add(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut sum = vec![0; a.len()];
    let mut carry = 0;
    for i in (0..a.len()).rev() {
        let digit = u16::from(a[i]) + u16::from(b[i]) + carry;
        sum[i] = digit as u8;
        carry = digit >> 8;
    }
    assert_eq!(carry, 0, "sum overflows the block");
    sum
}

#[test]
fn rejects_rsa_signatures_that_are_not_below_the_modulus() {
    // RFC 8017 §5.2.2: s must be below n. s + n has the same residue as a
    // valid s, and a verifier that reduces instead of rejecting (the `rsa`
    // crate's PSS verifier does) would accept it as a second signature. A
    // 2047-bit modulus leaves room for s + n in the 256-byte block.
    let modulus = left_pad(&rsa_modulus(&cert("rsa2047")), 256);
    for (kind, algorithm) in [("pkcs1", RsaPkcs1v15), ("pss", RsaPss)] {
        for hash in HashAlgorithm::ALL {
            let valid = signature("rsa2047", kind, hash);
            for (label, bad) in [("n", modulus.clone()), ("s + n", add(&valid, &modulus))] {
                assert_eq!(
                    verify_fixture("rsa2047", hash, algorithm, &bad),
                    Err(VerifyError::InvalidSignature),
                    "{kind} {hash}: {label}"
                );
            }
        }
    }
}

#[test]
fn rejects_ecdsa_with_a_zero_or_out_of_range_component() {
    let good = signature("p256", "ecdsa", Sha256);
    let (r, s) = good.split_at(32);
    let order = hex_decode("FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551");
    let one = [vec![0; 31], vec![1]].concat();
    let cases = [
        ("r = 0", [vec![0; 32], s.to_vec()].concat()),
        ("s = 0", [r.to_vec(), vec![0; 32]].concat()),
        ("r = n", [order.clone(), s.to_vec()].concat()),
        ("s = n", [r.to_vec(), order.clone()].concat()),
        ("r = 2^256-1", [vec![0xFF; 32], s.to_vec()].concat()),
        ("s = 2^256-1", [r.to_vec(), vec![0xFF; 32]].concat()),
        ("r = s = 1", [one.clone(), one].concat()),
        ("swapped r and s", [s, r].concat()),
    ];
    for (label, sig) in cases {
        assert_eq!(
            verify_fixture("p256", Sha256, Ecdsa, &sig),
            Err(VerifyError::InvalidSignature),
            "{label}"
        );
    }
}

#[test]
fn rejects_a_signature_from_another_key_of_the_same_kind() {
    let cases = [
        ("rsa2048", "rsa2048b", RsaPkcs1v15, "pkcs1"),
        ("rsa2048", "rsa2048b", RsaPss, "pss"),
        ("p256", "p256b", Ecdsa, "ecdsa"),
    ];
    for (cert_key, signer, algorithm, kind) in cases {
        for hash in HashAlgorithm::ALL {
            let sig = signature(signer, kind, hash);
            assert_eq!(
                verify_fixture(signer, hash, algorithm, &sig),
                Ok(()),
                "control: {signer} {kind} {hash}"
            );
            assert_eq!(
                verify_fixture(cert_key, hash, algorithm, &sig),
                Err(VerifyError::InvalidSignature),
                "{signer}'s {kind} against {cert_key} with {hash}"
            );
        }
    }
}

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

// --- error order ---------------------------------------------------------------------------------------

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

// --- VerifyError -----------------------------------------------------------------------------------------

#[test]
fn formats_each_error_variant() {
    let digest_err = Sha256.check_digest(&[0; 31]).unwrap_err();
    assert_eq!(
        VerifyError::Digest(digest_err).to_string(),
        "digest for SHA-256 must be 32 bytes, got 31"
    );
    assert_eq!(
        VerifyError::UnsupportedKey.to_string(),
        "unsupported public key"
    );
    assert_eq!(
        VerifyError::KeyMismatch(Ecdsa).to_string(),
        "ECDSA cannot be used with this key"
    );
    assert_eq!(
        VerifyError::KeyMismatch(RsaPss).to_string(),
        "RSASSA-PSS cannot be used with this key"
    );
    assert_eq!(
        VerifyError::KeyMismatch(RsaPkcs1v15).to_string(),
        "RSASSA-PKCS1-v1_5 cannot be used with this key"
    );
    assert_eq!(
        VerifyError::InvalidSignature.to_string(),
        "invalid signature"
    );
    let cert_err = CertError::Malformed("bad".to_owned());
    assert_eq!(
        VerifyError::Certificate(cert_err.clone()).to_string(),
        cert_err.to_string()
    );
}

#[test]
fn converts_from_the_errors_it_wraps() {
    let cert_err = CertError::Malformed("x".to_owned());
    assert_eq!(
        VerifyError::from(cert_err.clone()),
        VerifyError::Certificate(cert_err)
    );
    let digest_err = Sha512.check_digest(&[]).unwrap_err();
    assert_eq!(
        VerifyError::from(digest_err.clone()),
        VerifyError::Digest(digest_err)
    );
}

#[test]
fn verify_error_is_a_cloneable_comparable_std_error() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<VerifyError>();
    assert_ne!(VerifyError::UnsupportedKey, VerifyError::InvalidSignature);
    assert_ne!(
        VerifyError::KeyMismatch(Ecdsa),
        VerifyError::KeyMismatch(RsaPss)
    );
}

#[test]
fn a_certificate_that_is_only_a_prefix_never_panics_verification() {
    let full = cert("rsa2048");
    let digest = fixture_digest(Sha256);
    let sig = signature("rsa2048", "pkcs1", Sha256);
    for len in (0..full.len()).step_by(7) {
        assert!(
            verify(&full[..len], Sha256, RsaPkcs1v15, &digest, &sig).is_err(),
            "prefix of {len} bytes"
        );
    }
}
