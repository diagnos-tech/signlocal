//! SPEC §1: `HashAlgorithm`, `DigestLengthError`.

mod common;

use std::collections::HashSet;

use common::{FIXTURE_MESSAGE, fixture_digest, hex_decode};
use probe_core::{DigestLengthError, HashAlgorithm, UnknownAlgorithmError};

use HashAlgorithm::{Sha256, Sha384, Sha512};

fn repeated_a(count: usize) -> Vec<u8> {
    vec![b'a'; count]
}

// --- constants -------------------------------------------------------------

#[test]
fn lists_all_algorithms_from_shortest_digest_to_longest() {
    assert_eq!(HashAlgorithm::ALL, [Sha256, Sha384, Sha512]);
}

#[test]
fn reports_digest_length_in_bytes() {
    assert_eq!(Sha256.digest_len(), 32);
    assert_eq!(Sha384.digest_len(), 48);
    assert_eq!(Sha512.digest_len(), 64);
}

#[test]
fn reports_webcrypto_names() {
    assert_eq!(Sha256.name(), "SHA-256");
    assert_eq!(Sha384.name(), "SHA-384");
    assert_eq!(Sha512.name(), "SHA-512");
}

#[test]
fn displays_the_webcrypto_name() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(hash.to_string(), hash.name());
        assert_eq!(format!("{hash}"), hash.name());
    }
}

#[test]
fn is_copy_eq_and_hashable() {
    let copy = Sha256;
    let again = copy;
    assert_eq!(copy, again);
    assert_ne!(Sha256, Sha384);
    let set: HashSet<HashAlgorithm> = HashAlgorithm::ALL.into_iter().collect();
    assert_eq!(set.len(), 3);
}

// --- digest ----------------------------------------------------------------

/// (hash, input, expected hex) — expected values computed with `openssl dgst`.
#[test]
fn digests_match_known_answers() {
    let cases: [(HashAlgorithm, Vec<u8>, &str); 15] = [
        (
            Sha256,
            b"".to_vec(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            Sha256,
            b"abc".to_vec(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            Sha256,
            repeated_a(112),
            "f54353008a2553262ecdc4a34749563ba0950e8b0fc8652780b0a614b99683c1",
        ),
        (
            Sha256,
            repeated_a(128),
            "6836cf13bac400e9105071cd6af47084dfacad4e5e302c94bfed24e013afb73e",
        ),
        (
            Sha256,
            repeated_a(1_000_000),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
        ),
        (
            Sha384,
            b"".to_vec(),
            "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
        ),
        (
            Sha384,
            b"abc".to_vec(),
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7",
        ),
        (
            Sha384,
            repeated_a(112),
            "187d4e07cb306103c69967bf544d0dfbe9042577599c73c330abc0cb64c61236d5ed565ee19119d8c31779a38f791fcd",
        ),
        (
            Sha384,
            repeated_a(128),
            "edb12730a366098b3b2beac75a3bef1b0969b15c48e2163c23d96994f8d1bef760c7e27f3c464d3829f56c0d53808b0b",
        ),
        (
            Sha384,
            repeated_a(1_000_000),
            "9d0e1809716474cb086e834e310a4a1ced149e9c00f248527972cec5704c2a5b07b8b3dc38ecc4ebae97ddd87f3d8985",
        ),
        (
            Sha512,
            b"".to_vec(),
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        ),
        (
            Sha512,
            b"abc".to_vec(),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
        ),
        (
            Sha512,
            repeated_a(112),
            "c01d080efd492776a1c43bd23dd99d0a2e626d481e16782e75d54c2503b5dc32bd05f0f1ba33e568b88fd2d970929b719ecbb152f58f130a407c8830604b70ca",
        ),
        (
            Sha512,
            repeated_a(128),
            "b73d1929aa615934e61a871596b3f3b33359f42b8175602e89f7e06e5f658a243667807ed300314b95cacdd579f3e33abdfbe351909519a846d465c59582f321",
        ),
        (
            Sha512,
            repeated_a(1_000_000),
            "e718483d0ce769644e2e42c7bc15b4638e1f98b13b2044285632a803afa973ebde0ff244877ea60a4cb0432ce577c31beb009c5c2c49aa2e4eadb217ad8cc09b",
        ),
    ];
    for (hash, input, expected) in cases {
        assert_eq!(
            hash.digest(&input),
            hex_decode(expected),
            "{hash} over {} bytes",
            input.len()
        );
    }
}

#[test]
fn digests_the_fixture_message_like_openssl() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(hash.digest(FIXTURE_MESSAGE), fixture_digest(hash), "{hash}");
    }
}

#[test]
fn digest_output_has_the_declared_length_for_any_input_size() {
    for hash in HashAlgorithm::ALL {
        for size in [0, 1, 55, 56, 63, 64, 65, 111, 112, 127, 128, 129, 1000] {
            assert_eq!(
                hash.digest(&vec![7u8; size]).len(),
                hash.digest_len(),
                "{hash}/{size}"
            );
        }
    }
}

#[test]
fn digest_is_deterministic_and_input_sensitive() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(hash.digest(b"same"), hash.digest(b"same"));
        assert_ne!(hash.digest(b"same"), hash.digest(b"Same"));
        assert_ne!(hash.digest(b""), hash.digest(&[0]));
    }
}

#[test]
fn digest_passes_its_own_length_check() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(hash.check_digest(&hash.digest(b"payload")), Ok(()));
    }
}

// --- check_digest ----------------------------------------------------------

#[test]
fn accepts_digest_of_exactly_the_declared_length() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(
            hash.check_digest(&vec![0; hash.digest_len()]),
            Ok(()),
            "{hash}"
        );
        assert_eq!(
            hash.check_digest(&vec![0xFF; hash.digest_len()]),
            Ok(()),
            "{hash}"
        );
    }
}

#[test]
fn rejects_every_length_but_the_declared_one() {
    for hash in HashAlgorithm::ALL {
        for len in 0..=130 {
            let result = hash.check_digest(&vec![0x5A; len]);
            if len == hash.digest_len() {
                assert_eq!(result, Ok(()), "{hash} with {len} bytes");
            } else {
                assert_eq!(
                    result,
                    Err(DigestLengthError {
                        algorithm: hash,
                        expected: hash.digest_len(),
                        actual: len,
                    }),
                    "{hash} with {len} bytes"
                );
            }
        }
    }
}

#[test]
fn rejects_empty_digest() {
    let err = Sha256.check_digest(&[]).unwrap_err();
    assert_eq!(err.actual, 0);
    assert_eq!(err.expected, 32);
}

#[test]
fn rejects_digest_shorter_than_hash() {
    let err = Sha384.check_digest(&[0; 47]).unwrap_err();
    assert_eq!((err.algorithm, err.expected, err.actual), (Sha384, 48, 47));
}

#[test]
fn rejects_digest_longer_than_hash() {
    let err = Sha512.check_digest(&[0; 65]).unwrap_err();
    assert_eq!((err.algorithm, err.expected, err.actual), (Sha512, 64, 65));
}

#[test]
fn rejects_digest_of_another_hash() {
    for hash in HashAlgorithm::ALL {
        for other in HashAlgorithm::ALL {
            let digest = other.digest(b"x");
            assert_eq!(
                hash.check_digest(&digest).is_ok(),
                hash == other,
                "{hash} vs {other}"
            );
        }
    }
}

// --- DigestLengthError -----------------------------------------------------

#[test]
fn formats_digest_length_error() {
    let err = Sha256.check_digest(&[0; 31]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "digest for SHA-256 must be 32 bytes, got 31"
    );
    let err = Sha384.check_digest(&[0; 32]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "digest for SHA-384 must be 48 bytes, got 32"
    );
    let err = Sha512.check_digest(&[]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "digest for SHA-512 must be 64 bytes, got 0"
    );
}

#[test]
fn digest_length_error_is_a_cloneable_comparable_std_error() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<DigestLengthError>();
    let err = Sha256.check_digest(&[0; 3]).unwrap_err();
    assert_eq!(err.clone(), err);
    assert!(format!("{err:?}").contains("DigestLengthError"));
}

// --- FromStr ---------------------------------------------------------------

#[test]
fn parses_every_accepted_spelling_ignoring_case() {
    let cases = [
        ("SHA-256", Sha256),
        ("SHA256", Sha256),
        ("sha-256", Sha256),
        ("sha256", Sha256),
        ("Sha-256", Sha256),
        ("sHa256", Sha256),
        ("SHA-384", Sha384),
        ("SHA384", Sha384),
        ("sha-384", Sha384),
        ("sha384", Sha384),
        ("SHA-512", Sha512),
        ("SHA512", Sha512),
        ("sha-512", Sha512),
        ("sha512", Sha512),
        ("ShA-512", Sha512),
    ];
    for (text, expected) in cases {
        assert_eq!(text.parse::<HashAlgorithm>(), Ok(expected), "{text:?}");
    }
}

#[test]
fn rejects_everything_else_with_the_original_string() {
    let rejected = [
        "",
        " ",
        " SHA-256",
        "SHA-256 ",
        " SHA-256 ",
        "SHA-256\n",
        "\tSHA-256",
        "SHA-1",
        "SHA1",
        "SHA-224",
        "SHA224",
        "SHA-512/256",
        "SHA512/256",
        "SHA3-256",
        "SHA3-512",
        "SHA_256",
        "SHA 256",
        "SHA--256",
        "SHA-2560",
        "SHA-25",
        "SHA-",
        "SHA",
        "256",
        "-256",
        "MD5",
        "sha256sum",
        "SHA-256\0",
        "SHA\u{2011}256",
        "ＳＨＡ-256",
        "SHA-２５６",
        "ECDSA",
        "RSASSA-PSS",
    ];
    for text in rejected {
        assert_eq!(
            text.parse::<HashAlgorithm>(),
            Err(UnknownAlgorithmError {
                name: text.to_owned()
            }),
            "{text:?}"
        );
    }
}

#[test]
fn case_folding_is_ascii_only() {
    // Case-insensitive means ASCII case-insensitive.
    // U+017F (long s) upper-cases to `S` under full Unicode rules and must
    // not sneak in as an `s`.
    assert!("ſha-256".parse::<HashAlgorithm>().is_err());
    assert!("ſha512".parse::<HashAlgorithm>().is_err());
}

#[test]
fn unknown_hash_error_displays_the_offending_name() {
    let err = "SHA-1".parse::<HashAlgorithm>().unwrap_err();
    assert_eq!(err.to_string(), "unknown algorithm: SHA-1");
    let err = " SHA-256".parse::<HashAlgorithm>().unwrap_err();
    assert_eq!(err.to_string(), "unknown algorithm:  SHA-256");
    let err = "".parse::<HashAlgorithm>().unwrap_err();
    assert_eq!(err.to_string(), "unknown algorithm: ");
}

#[test]
fn round_trips_name_and_display_through_from_str() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(hash.name().parse::<HashAlgorithm>(), Ok(hash));
        assert_eq!(hash.to_string().parse::<HashAlgorithm>(), Ok(hash));
        assert_eq!(
            hash.name().to_lowercase().parse::<HashAlgorithm>(),
            Ok(hash)
        );
    }
}

#[test]
fn does_not_accept_signature_algorithm_names() {
    for text in ["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"] {
        assert!(text.parse::<HashAlgorithm>().is_err(), "{text}");
    }
}
