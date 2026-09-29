//! Keys that the `rsa` and `p256`/`p384`/`p521` crates must refuse, reported
//! as `UnsupportedKey` and never as a panic. Valid and tampered signatures
//! are covered by the integration tests over OpenSSL fixtures.

use super::*;
use crate::testkit::{TestCert, spki_ec_point, spki_rsa_integers};

const DIGEST: [u8; 32] = [0x42; 32];
const F4: [u8; 3] = [0x01, 0x00, 0x01];

fn rsa_cert(modulus: &[u8], exponent: &[u8]) -> Vec<u8> {
    TestCert::new()
        .spki(spki_rsa_integers(modulus, exponent))
        .build()
}

/// A positive INTEGER content of `bits` bits, all ones except `last`.
fn modulus(bits: usize, last: u8) -> Vec<u8> {
    let mut value = vec![0xff; bits.div_ceil(8)];
    value[0] >>= value.len() * 8 - bits;
    *value.last_mut().unwrap() = last;
    [vec![0x00], value].concat()
}

#[test]
fn rsa_keys_the_verifier_cannot_use_are_unsupported() {
    let odd = modulus(2048, 0xff);
    let cases: [(&str, Vec<u8>, Vec<u8>); 9] = [
        ("zero modulus", vec![0x00], F4.to_vec()),
        ("modulus one", vec![0x01], vec![0x03]),
        ("even modulus", modulus(2048, 0xfe), F4.to_vec()),
        ("modulus above 8192 bits", modulus(8200, 0xff), F4.to_vec()),
        ("zero exponent", odd.clone(), vec![0x00]),
        ("exponent one", odd.clone(), vec![0x01]),
        ("even exponent", odd.clone(), vec![0x01, 0x00, 0x00]),
        (
            "exponent above 2^33",
            odd.clone(),
            vec![0x02, 0x00, 0x00, 0x00, 0x01],
        ),
        ("exponent not below the modulus", odd.clone(), odd.clone()),
    ];
    for (label, n, e) in cases {
        let cert = rsa_cert(&n, &e);
        for algorithm in [SignatureAlgorithm::RsaPkcs1v15, SignatureAlgorithm::RsaPss] {
            for signature in [vec![], vec![0x01; n.len() - 1], vec![0xff; 2048]] {
                assert_eq!(
                    verify(&cert, HashAlgorithm::Sha256, algorithm, &DIGEST, &signature),
                    Err(VerifyError::UnsupportedKey),
                    "{label}"
                );
            }
        }
    }
}

#[test]
fn a_tiny_but_valid_rsa_key_only_yields_invalid_signatures() {
    // n = 0xfd = 253 (odd), e = 3: accepted as a key, far too small to hold
    // any encoded digest.
    let cert = rsa_cert(&[0x00, 0xfd], &[0x03]);
    for algorithm in [SignatureAlgorithm::RsaPkcs1v15, SignatureAlgorithm::RsaPss] {
        for signature in [
            vec![0x00],
            vec![0x01],
            vec![0xfc],
            vec![0xfd],
            vec![0x01, 0x02],
        ] {
            assert_eq!(
                verify(&cert, HashAlgorithm::Sha256, algorithm, &DIGEST, &signature),
                Err(VerifyError::InvalidSignature),
                "{signature:02x?}"
            );
        }
    }
}

#[test]
fn ec_points_that_are_not_on_their_curve_are_unsupported() {
    let curves = [
        ("1.2.840.10045.3.1.7", 32),
        ("1.3.132.0.34", 48),
        ("1.3.132.0.35", 66),
    ];
    for (curve, field_len) in curves {
        let uncompressed = |fill: u8| [vec![0x04], vec![fill; 2 * field_len]].concat();
        let points = [
            ("empty", vec![]),
            ("identity", vec![0x00]),
            ("off the curve", uncompressed(0x11)),
            ("origin", uncompressed(0x00)),
            ("coordinates beyond the field", uncompressed(0xff)),
            (
                "compressed x beyond the field",
                [vec![0x02], vec![0xff; field_len]].concat(),
            ),
            ("one byte short", uncompressed(0x11)[1..].to_vec()),
            (
                "a point of another size",
                [vec![0x04], vec![0x11; 2 * field_len + 2]].concat(),
            ),
        ];
        for (label, point) in points {
            let cert = TestCert::new().spki(spki_ec_point(curve, &point)).build();
            let signature = vec![0x01; 2 * field_len];
            assert_eq!(
                verify(
                    &cert,
                    HashAlgorithm::Sha256,
                    SignatureAlgorithm::Ecdsa,
                    &DIGEST,
                    &signature
                ),
                Err(VerifyError::UnsupportedKey),
                "{curve}: {label}"
            );
        }
    }
}
