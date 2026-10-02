//! SPEC §3: `pkcs1::digest_info`.

mod common;

use common::{counting_bytes, fixture_digest, fixture_digest_info, hex_decode};
use probe_core::pkcs1::digest_info;
use probe_core::{DigestLengthError, HashAlgorithm};

use HashAlgorithm::{Sha256, Sha384, Sha512};

/// The prefixes exactly as SPEC §3 lists them.
fn spec_prefix(hash: HashAlgorithm) -> Vec<u8> {
    hex_decode(match hash {
        Sha256 => "3031300d060960864801650304020105000420",
        Sha384 => "3041300d060960864801650304020205000430",
        Sha512 => "3051300d060960864801650304020305000440",
    })
}

fn sample_digests(hash: HashAlgorithm) -> Vec<Vec<u8>> {
    let len = hash.digest_len();
    vec![
        vec![0x00; len],
        vec![0xFF; len],
        vec![0x30; len],
        counting_bytes(1, len),
        counting_bytes(0xF0, len),
        fixture_digest(hash),
    ]
}

#[test]
fn matches_the_digest_info_built_by_openssl() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(
            digest_info(hash, &fixture_digest(hash)),
            Ok(fixture_digest_info(hash)),
            "{hash}"
        );
    }
}

#[test]
fn prepends_the_exact_prefix_of_each_hash() {
    for hash in HashAlgorithm::ALL {
        let prefix = spec_prefix(hash);
        for digest in sample_digests(hash) {
            let out = digest_info(hash, &digest).expect("valid digest");
            assert_eq!(out[..prefix.len()], prefix[..], "{hash}");
        }
    }
}

#[test]
fn appends_the_digest_unchanged() {
    for hash in HashAlgorithm::ALL {
        let prefix_len = spec_prefix(hash).len();
        for digest in sample_digests(hash) {
            let out = digest_info(hash, &digest).expect("valid digest");
            assert_eq!(out[prefix_len..], digest[..], "{hash}");
        }
    }
}

#[test]
fn produces_prefix_plus_digest_and_nothing_else() {
    assert_eq!(digest_info(Sha256, &[0; 32]).unwrap().len(), 19 + 32);
    assert_eq!(digest_info(Sha384, &[0; 48]).unwrap().len(), 19 + 48);
    assert_eq!(digest_info(Sha512, &[0; 64]).unwrap().len(), 19 + 64);
}

#[test]
fn builds_a_well_formed_outer_sequence() {
    for hash in HashAlgorithm::ALL {
        let out = digest_info(hash, &vec![0xAA; hash.digest_len()]).unwrap();
        assert_eq!(out[0], 0x30, "{hash}: SEQUENCE tag");
        assert_eq!(
            usize::from(out[1]),
            out.len() - 2,
            "{hash}: short-form length"
        );
        assert_eq!(out[2..4], [0x30, 0x0D], "{hash}: AlgorithmIdentifier");
        assert_eq!(
            out[out.len() - hash.digest_len() - 2],
            0x04,
            "{hash}: OCTET STRING tag"
        );
        assert_eq!(
            usize::from(out[out.len() - hash.digest_len() - 1]),
            hash.digest_len(),
            "{hash}: OCTET STRING length"
        );
    }
}

#[test]
fn output_differs_per_hash_even_for_equal_byte_patterns() {
    let a = digest_info(Sha256, &[7; 32]).unwrap();
    let b = digest_info(Sha384, &[7; 48]).unwrap();
    let c = digest_info(Sha512, &[7; 64]).unwrap();
    assert_ne!(a[..19], b[..19]);
    assert_ne!(b[..19], c[..19]);
    assert_ne!(a[..19], c[..19]);
}

#[test]
fn is_deterministic() {
    let digest = Sha256.digest(b"payload");
    assert_eq!(digest_info(Sha256, &digest), digest_info(Sha256, &digest));
}

#[test]
fn rejects_digest_of_the_wrong_length_with_the_hash_check_error() {
    for hash in HashAlgorithm::ALL {
        for len in [0, 1, 20, 31, 33, 47, 49, 63, 65, 100] {
            let result = digest_info(hash, &vec![0x11; len]);
            if len == hash.digest_len() {
                assert!(result.is_ok(), "{hash}/{len}");
            } else {
                assert_eq!(
                    result,
                    Err(DigestLengthError {
                        algorithm: hash,
                        expected: hash.digest_len(),
                        actual: len,
                    }),
                    "{hash}/{len}"
                );
            }
        }
    }
}

#[test]
fn rejects_an_empty_digest() {
    let err = digest_info(Sha256, &[]).unwrap_err();
    assert_eq!((err.algorithm, err.expected, err.actual), (Sha256, 32, 0));
}

#[test]
fn rejects_a_digest_made_with_another_hash() {
    let sha384 = Sha384.digest(b"x");
    let err = digest_info(Sha256, &sha384).unwrap_err();
    assert_eq!((err.algorithm, err.expected, err.actual), (Sha256, 32, 48));
    let sha256 = Sha256.digest(b"x");
    let err = digest_info(Sha512, &sha256).unwrap_err();
    assert_eq!((err.algorithm, err.expected, err.actual), (Sha512, 64, 32));
}

#[test]
fn reports_the_same_error_as_check_digest() {
    for hash in HashAlgorithm::ALL {
        for len in [0, 5, 40, 70] {
            let digest = vec![0x22; len];
            assert_eq!(
                digest_info(hash, &digest).map(|_| ()),
                hash.check_digest(&digest),
                "{hash}/{len}"
            );
        }
    }
}
