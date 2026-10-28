//! Digest.

use super::*;

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
