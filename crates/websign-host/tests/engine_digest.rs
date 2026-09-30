//! SPEC §8.6 and T4: the digest must have exactly the length of the hash the
//! request declared.

mod common;

use common::harness::{Harness, hash_str};
use common::{Cert, ORIGIN, wire};
use websign_core::HashAlgorithm;
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::port::Failure;

const HASHES: [HashAlgorithm; 3] = [
    HashAlgorithm::Sha256,
    HashAlgorithm::Sha384,
    HashAlgorithm::Sha512,
];

fn awaiting(hash: HashAlgorithm) -> (Harness, Cert) {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.begin("s1", ORIGIN, hash);
    let out = h.listed(&[&p]);
    assert_eq!(out.need_digests().len(), 1);
    (h, p)
}

#[test]
fn the_declared_hash_appears_in_need_digest() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.send(wire::sign_begin(
        "s1",
        Some(ORIGIN),
        hash_str(HashAlgorithm::Sha384),
    ));
    h.take();
    let out = h.listed(&[&p]);
    assert_eq!(
        serde_json::to_string(&out.need_digests()[0].1.hash).expect("json"),
        "\"SHA-384\""
    );
}

#[test]
fn an_exact_length_digest_is_accepted_for_every_hash() {
    for hash in HASHES {
        let (mut h, _) = awaiting(hash);
        let out = h.answer_digest("s1", 1, hash);
        assert_eq!(out.digest_ready_count(), 1, "{hash:?}");
        assert!(out.errors().is_empty(), "{hash:?}");
    }
}

#[test]
fn a_digest_one_byte_off_ends_the_request_with_invalid_request() {
    for hash in HASHES {
        for len in [hash.digest_len() - 1, hash.digest_len() + 1, 20] {
            let (mut h, _) = awaiting(hash);
            h.send(wire::digest("s1", 1, &vec![0xAB; len]));
            let out = h.take();
            assert_eq!(
                out.only_error(),
                ("s1".to_owned(), ErrorCode::InvalidRequest),
                "{hash:?} with {len} bytes"
            );
            assert_eq!(out.digest_ready_count(), 0);
            assert!(out.signs().is_empty());
            assert!(
                out.failures()
                    .iter()
                    .any(|f| matches!(f, Failure::Internal { .. })),
                "the window is told it was the site's bug"
            );
        }
    }
}

#[test]
fn another_hashs_digest_length_is_wrong_too() {
    let (mut h, _) = awaiting(HashAlgorithm::Sha256);
    let out = h.answer_digest("s1", 1, HashAlgorithm::Sha512);
    assert_eq!(out.only_error().1, ErrorCode::InvalidRequest);
}

#[test]
fn the_request_is_over_after_a_wrong_length() {
    let (mut h, _) = awaiting(HashAlgorithm::Sha256);
    h.send(wire::digest("s1", 1, &[1; 31]));
    h.take();
    // A correct digest afterwards refers to a request that no longer exists.
    let out = h.answer_digest("s1", 1, HashAlgorithm::Sha256);
    assert_eq!(out.only_error().1, ErrorCode::InvalidRequest);
    assert_eq!(out.digest_ready_count(), 0);
}

#[test]
fn a_digest_for_an_id_that_is_not_open_is_invalid() {
    let mut h = Harness::native_ready();
    let out = h.answer_digest("nope", 1, HashAlgorithm::Sha256);
    assert_eq!(
        out.only_error(),
        ("nope".to_owned(), ErrorCode::InvalidRequest)
    );
}

#[test]
fn the_base64_of_a_digest_is_strict() {
    let (mut h, _) = awaiting(HashAlgorithm::Sha256);
    // Not canonical padded Base64: refused by the parser, addressed to the id.
    let raw = br#"{"v":1,"id":"s1","type":"sign.digest","seq":1,"digest":"AAAA*"}"#;
    h.send(raw.to_vec());
    assert_eq!(
        h.take().only_error(),
        ("s1".to_owned(), ErrorCode::InvalidRequest)
    );
}
