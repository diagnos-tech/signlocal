use super::fixtures::*;
use super::*;
use crate::cert::PublicKeyKind;
use crate::ecdsa::{Curve, der_to_raw};

const PKCS1: SignatureAlgorithm = SignatureAlgorithm::RsaPkcs1v15;
const PSS: SignatureAlgorithm = SignatureAlgorithm::RsaPss;
const ECDSA: SignatureAlgorithm = SignatureAlgorithm::Ecdsa;

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("fixture hex"))
        .collect()
}

fn digest(hash: HashAlgorithm) -> Vec<u8> {
    hash.digest(MESSAGE)
}

fn rsa_cert() -> Vec<u8> {
    unhex(RSA_CERT)
}

/// (curve, certificate, DER signatures per hash).
fn ec_fixtures() -> [(Curve, Vec<u8>, &'static [&'static str; 3]); 3] {
    [
        (Curve::P256, unhex(P256_CERT), &P256_DER_SIGNATURES),
        (Curve::P384, unhex(P384_CERT), &P384_DER_SIGNATURES),
        (Curve::P521, unhex(P521_CERT), &P521_DER_SIGNATURES),
    ]
}

fn raw_ec_signature(curve: Curve, der_hex: &str) -> Vec<u8> {
    der_to_raw(&unhex(der_hex), curve).expect("fixture signature")
}

#[test]
fn fixtures_have_the_key_types_they_claim() {
    let kind = |der: &[u8]| CertInfo::from_der(der).unwrap().key;
    assert_eq!(kind(&rsa_cert()), PublicKeyKind::Rsa { bits: 2048 });
    for (curve, cert, _) in ec_fixtures() {
        assert_eq!(kind(&cert), PublicKeyKind::Ec { curve });
    }
    assert!(matches!(
        kind(&unhex(ED25519_CERT)),
        PublicKeyKind::Unsupported { .. }
    ));
    assert_eq!(
        kind(&unhex(SECP256K1_CERT)),
        PublicKeyKind::Unsupported {
            oid: "1.3.132.0.10".into()
        }
    );
}

#[test]
fn accepts_rsa_pkcs1_v1_5_signatures_for_every_hash() {
    for (hash, sig) in HashAlgorithm::ALL.into_iter().zip(RSA_PKCS1_SIGNATURES) {
        assert_eq!(
            verify(&rsa_cert(), hash, PKCS1, &digest(hash), &unhex(sig)),
            Ok(()),
            "{hash}"
        );
    }
}

#[test]
fn accepts_rsa_pss_signatures_for_every_hash() {
    for (hash, sig) in HashAlgorithm::ALL.into_iter().zip(RSA_PSS_SIGNATURES) {
        assert_eq!(
            verify(&rsa_cert(), hash, PSS, &digest(hash), &unhex(sig)),
            Ok(()),
            "{hash}"
        );
    }
}

#[test]
fn accepts_ecdsa_on_every_curve_with_every_hash() {
    for (curve, cert, signatures) in ec_fixtures() {
        for (hash, sig) in HashAlgorithm::ALL.into_iter().zip(signatures) {
            let raw = raw_ec_signature(curve, sig);
            assert_eq!(raw.len(), curve.signature_len());
            assert_eq!(
                verify(&cert, hash, ECDSA, &digest(hash), &raw),
                Ok(()),
                "{} with {hash}",
                curve.name()
            );
        }
    }
}

#[test]
fn rsa_signatures_are_not_interchangeable_between_schemes() {
    let hash = HashAlgorithm::Sha256;
    let pkcs1 = unhex(RSA_PKCS1_SIGNATURES[0]);
    let pss = unhex(RSA_PSS_SIGNATURES[0]);
    assert_eq!(
        verify(&rsa_cert(), hash, PSS, &digest(hash), &pkcs1),
        Err(VerifyError::InvalidSignature)
    );
    assert_eq!(
        verify(&rsa_cert(), hash, PKCS1, &digest(hash), &pss),
        Err(VerifyError::InvalidSignature)
    );
}

#[test]
fn a_signature_not_below_the_modulus_is_rejected_even_if_congruent() {
    let hash = HashAlgorithm::Sha256;
    let digest = digest(hash);
    let canonical = unhex(RSA_PSS_SHA256_CANONICAL);
    let twin = unhex(RSA_PSS_SHA256_PLUS_MODULUS);
    assert_eq!(canonical.len(), twin.len());
    assert_eq!(verify(&rsa_cert(), hash, PSS, &digest, &canonical), Ok(()));
    assert_eq!(
        verify(&rsa_cert(), hash, PSS, &digest, &twin),
        Err(VerifyError::InvalidSignature)
    );
}

#[test]
fn a_signature_made_for_another_hash_is_rejected() {
    // SHA-256 signature checked as if it covered a SHA-512 digest of the same data.
    let sig = unhex(RSA_PKCS1_SIGNATURES[0]);
    let hash = HashAlgorithm::Sha512;
    assert_eq!(
        verify(&rsa_cert(), hash, PKCS1, &digest(hash), &sig),
        Err(VerifyError::InvalidSignature)
    );
}

#[test]
fn a_different_digest_is_rejected() {
    for hash in HashAlgorithm::ALL {
        let other = hash.digest(b"something else entirely");
        let sig = unhex(
            RSA_PKCS1_SIGNATURES[HashAlgorithm::ALL.iter().position(|h| *h == hash).unwrap()],
        );
        assert_eq!(
            verify(&rsa_cert(), hash, PKCS1, &other, &sig),
            Err(VerifyError::InvalidSignature)
        );
    }
    for (curve, cert, signatures) in ec_fixtures() {
        let hash = HashAlgorithm::Sha256;
        let raw = raw_ec_signature(curve, signatures[0]);
        let other = hash.digest(b"something else entirely");
        assert_eq!(
            verify(&cert, hash, ECDSA, &other, &raw),
            Err(VerifyError::InvalidSignature)
        );
    }
}

#[test]
fn any_flipped_bit_in_a_signature_is_rejected() {
    let hash = HashAlgorithm::Sha256;
    let rsa_sig = unhex(RSA_PKCS1_SIGNATURES[0]);
    for index in [0, 1, 100, rsa_sig.len() - 1] {
        let mut bad = rsa_sig.clone();
        bad[index] ^= 0x01;
        assert_eq!(
            verify(&rsa_cert(), hash, PKCS1, &digest(hash), &bad),
            Err(VerifyError::InvalidSignature)
        );
    }
    for (curve, cert, signatures) in ec_fixtures() {
        let raw = raw_ec_signature(curve, signatures[0]);
        for index in [0, curve.field_len() - 1, curve.field_len(), raw.len() - 1] {
            let mut bad = raw.clone();
            bad[index] ^= 0x01;
            assert_eq!(
                verify(&cert, hash, ECDSA, &digest(hash), &bad),
                Err(VerifyError::InvalidSignature)
            );
        }
    }
}

#[test]
fn wrong_signature_sizes_are_invalid_signatures() {
    let hash = HashAlgorithm::Sha256;
    let rsa_sig = unhex(RSA_PKCS1_SIGNATURES[0]);
    let cases = [
        rsa_sig[..rsa_sig.len() - 1].to_vec(),
        [rsa_sig.clone(), vec![0]].concat(),
        [vec![0], rsa_sig].concat(),
        Vec::new(),
        vec![0; 256],
    ];
    for algorithm in [PKCS1, PSS] {
        for sig in &cases {
            assert_eq!(
                verify(&rsa_cert(), hash, algorithm, &digest(hash), sig),
                Err(VerifyError::InvalidSignature)
            );
        }
    }

    for (curve, cert, signatures) in ec_fixtures() {
        let raw = raw_ec_signature(curve, signatures[0]);
        let der = unhex(signatures[0]);
        let cases = [
            raw[..raw.len() - 1].to_vec(),
            [raw.clone(), vec![0]].concat(),
            der,
            Vec::new(),
            vec![0; curve.signature_len()],
            vec![0xff; curve.signature_len()],
        ];
        for sig in cases {
            assert_eq!(
                verify(&cert, hash, ECDSA, &digest(hash), &sig),
                Err(VerifyError::InvalidSignature),
                "{} with {} bytes",
                curve.name(),
                sig.len()
            );
        }
    }
}

#[test]
fn checks_run_in_the_documented_order() {
    let hash = HashAlgorithm::Sha256;
    let short_digest = [0u8; 31];
    let sig = unhex(RSA_PKCS1_SIGNATURES[0]);

    // 1. certificate before everything
    let garbage = b"not a certificate";
    assert!(matches!(
        verify(garbage, hash, PKCS1, &short_digest, &sig),
        Err(VerifyError::Certificate(CertError::Malformed(_)))
    ));

    // 2. digest length before key checks
    let ed25519 = unhex(ED25519_CERT);
    assert_eq!(
        verify(&ed25519, hash, PKCS1, &short_digest, &sig),
        Err(VerifyError::Digest(DigestLengthError {
            algorithm: hash,
            expected: 32,
            actual: 31
        }))
    );
    assert!(matches!(
        verify(
            &rsa_cert(),
            HashAlgorithm::Sha384,
            PKCS1,
            &digest(hash),
            &sig
        ),
        Err(VerifyError::Digest(DigestLengthError {
            expected: 48,
            actual: 32,
            ..
        }))
    ));

    // 3. unsupported key before algorithm mismatch
    for cert in [ed25519, unhex(SECP256K1_CERT)] {
        for algorithm in SignatureAlgorithm::ALL {
            assert_eq!(
                verify(&cert, hash, algorithm, &digest(hash), &sig),
                Err(VerifyError::UnsupportedKey)
            );
        }
    }

    // 4. algorithm mismatch before the signature is looked at
    assert_eq!(
        verify(&rsa_cert(), hash, ECDSA, &digest(hash), &[]),
        Err(VerifyError::KeyMismatch(ECDSA))
    );
    for (_, cert, _) in ec_fixtures() {
        for algorithm in [PKCS1, PSS] {
            assert_eq!(
                verify(&cert, hash, algorithm, &digest(hash), &[]),
                Err(VerifyError::KeyMismatch(algorithm))
            );
        }
    }
}

#[test]
fn error_messages_are_readable() {
    assert_eq!(
        VerifyError::UnsupportedKey.to_string(),
        "unsupported public key"
    );
    assert_eq!(
        VerifyError::InvalidSignature.to_string(),
        "invalid signature"
    );
    assert_eq!(
        VerifyError::KeyMismatch(ECDSA).to_string(),
        "ECDSA cannot be used with this key"
    );
    assert_eq!(
        VerifyError::Digest(DigestLengthError {
            algorithm: HashAlgorithm::Sha256,
            expected: 32,
            actual: 31
        })
        .to_string(),
        "digest for SHA-256 must be 32 bytes, got 31"
    );
}

#[test]
fn arbitrary_signatures_never_panic() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let rsa = rsa_cert();
    let ecs = ec_fixtures();
    for round in 0..60 {
        let hash = HashAlgorithm::ALL[round % 3];
        let digest = digest(hash);
        let mut random = |len: usize| (0..len).map(|_| next() as u8).collect::<Vec<u8>>();
        for algorithm in [PKCS1, PSS] {
            let _ = verify(&rsa, hash, algorithm, &digest, &random(256));
            let _ = verify(&rsa, hash, algorithm, &digest, &random(round % 300));
        }
        for (curve, cert, _) in &ecs {
            let _ = verify(cert, hash, ECDSA, &digest, &random(curve.signature_len()));
        }
    }
}

#[test]
fn corrupted_certificates_never_panic() {
    let hash = HashAlgorithm::Sha256;
    let digest = digest(hash);
    let sig = unhex(RSA_PKCS1_SIGNATURES[0]);
    let cert = rsa_cert();
    for index in (0..cert.len()).step_by(7) {
        let mut mutated = cert.clone();
        mutated[index] ^= 0xff;
        let _ = verify(&mutated, hash, PKCS1, &digest, &sig);
    }
    let ec = unhex(P256_CERT);
    let raw = raw_ec_signature(Curve::P256, P256_DER_SIGNATURES[0]);
    for index in 0..ec.len() {
        let mut mutated = ec.clone();
        mutated[index] ^= 0xff;
        let _ = verify(&mutated, hash, ECDSA, &digest, &raw);
    }
}
