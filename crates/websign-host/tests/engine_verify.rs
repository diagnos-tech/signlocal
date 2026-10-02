//! SPEC §8.8, §4.1 and T8: what the key store returns is verified before it
//! reaches the page.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN};
use serde_json::json;
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_host::ports::KeyReply;
use websign_protocol::types::{HashName, SignatureAlgorithmName};
use websign_ui_model::confirm::port::Failure;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;
const ECDSA: SignatureAlgorithm = SignatureAlgorithm::Ecdsa;

fn signing(
    cert: &Cert,
    hash: HashAlgorithm,
    algorithms: &[&str],
) -> (Harness, u64, websign_ui_model::confirm::port::RequestKey) {
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[cert]);
    let frame = common::wire::sign_begin_with(
        "s1",
        ORIGIN,
        json!({ "hash": common::harness::hash_str(hash), "algorithms": algorithms }),
    );
    h.send(frame);
    let key = h.take().opens()[0].key;
    h.listed(&[cert]);
    h.answer_digest("s1", 1, hash);
    let tag = h.press_sign(key, cert, None).signs()[0].tag;
    (h, tag, key)
}

#[test]
fn garbage_from_the_key_store_is_not_sent_and_shows_a_driver_failure() {
    let p = Cert::p256();
    let (mut h, tag, _) = signing(&p, SHA256, &["ECDSA"]);
    let out = h.signed_ok(tag, &p, SHA256, ECDSA);
    assert_eq!(out.results().len(), 1, "control: a good signature passes");

    let (mut h, tag, _) = signing(&p, SHA256, &["ECDSA"]);
    h.keys(KeyReply::Signed {
        tag,
        result: Ok(vec![0x42; 64]),
    });
    let out = h.take();
    assert!(
        out.frames.is_empty(),
        "nothing reaches the page: {:?}",
        out.kinds()
    );
    match out.failures().as_slice() {
        [Failure::DriverFailure { native, .. }] => {
            assert_eq!(native, "signature did not verify");
        }
        other => panic!("expected one DriverFailure, got {other:?}"),
    }
    let recorded = h.state.borrow();
    assert_eq!(recorded.errors.len(), 1);
    assert_eq!(recorded.errors[0].code, "DriverFailure");
}

#[test]
fn a_signature_of_the_wrong_length_or_for_another_key_is_refused() {
    let p = Cert::p256();
    for bad in [vec![], vec![1; 63], vec![1; 65]] {
        let (mut h, tag, _) = signing(&p, SHA256, &["ECDSA"]);
        h.keys(KeyReply::Signed {
            tag,
            result: Ok(bad),
        });
        let out = h.take();
        assert!(out.results().is_empty());
        assert!(out.errors().is_empty());
    }
    // A valid signature, but of a different digest than the page sent.
    let (mut h, tag, _) = signing(&p, SHA256, &["ECDSA"]);
    let other_hash_signature = p.signature(HashAlgorithm::Sha384, ECDSA);
    h.keys(KeyReply::Signed {
        tag,
        result: Ok(other_hash_signature),
    });
    assert!(h.take().results().is_empty());
}

#[test]
fn after_a_failed_self_check_the_person_can_try_again() {
    let p = Cert::p256();
    let (mut h, tag, key) = signing(&p, SHA256, &["ECDSA"]);
    h.keys(KeyReply::Signed {
        tag,
        result: Ok(vec![0; 64]),
    });
    h.take();
    let out = h.press_sign(key, &p, None);
    let again = out.signs();
    assert_eq!(again.len(), 1, "state is Ready");
    let out = h.signed_ok(again[0].tag, &p, SHA256, ECDSA);
    assert_eq!(out.results().len(), 1);
}

#[test]
fn every_hash_and_algorithm_pair_the_fixtures_cover_signs_and_verifies() {
    let cases: [(Cert, &str, SignatureAlgorithm, SignatureAlgorithmName); 3] = [
        (Cert::p256(), "ECDSA", ECDSA, SignatureAlgorithmName::Ecdsa),
        (
            Cert::rsa(),
            "RSASSA-PKCS1-v1_5",
            SignatureAlgorithm::RsaPkcs1v15,
            SignatureAlgorithmName::RsaPkcs1v15,
        ),
        (
            Cert::rsa(),
            "RSASSA-PSS",
            SignatureAlgorithm::RsaPss,
            SignatureAlgorithmName::RsaPss,
        ),
    ];
    for (cert, wire_name, algorithm, named) in cases {
        for hash in [
            HashAlgorithm::Sha256,
            HashAlgorithm::Sha384,
            HashAlgorithm::Sha512,
        ] {
            let (mut h, tag, _) = signing(&cert, hash, &[wire_name]);
            let out = h.signed_ok(tag, &cert, hash, algorithm);
            let results = out.results();
            assert_eq!(results.len(), 1, "{wire_name} {hash:?}");
            assert_eq!(results[0].1.algorithm, named);
            assert_eq!(
                serde_json::to_value(results[0].1.hash).expect("json"),
                serde_json::to_value(match hash {
                    HashAlgorithm::Sha256 => HashName::Sha256,
                    HashAlgorithm::Sha384 => HashName::Sha384,
                    HashAlgorithm::Sha512 => HashName::Sha512,
                })
                .expect("json")
            );
            assert_eq!(
                results[0].1.signature.as_bytes(),
                cert.signature(hash, algorithm).as_slice()
            );
        }
    }
}

#[test]
fn the_key_command_carries_the_hash_and_algorithm_of_the_request() {
    let r = Cert::rsa();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&r]);
    let frame = common::wire::sign_begin_with(
        "s1",
        ORIGIN,
        json!({ "hash": "SHA-512", "algorithms": ["RSASSA-PSS"] }),
    );
    h.send(frame);
    let key = h.take().opens()[0].key;
    h.listed(&[&r]);
    h.answer_digest("s1", 1, HashAlgorithm::Sha512);
    let call = h.press_sign(key, &r, None).signs().remove(0);
    assert_eq!(call.hash, HashAlgorithm::Sha512);
    assert_eq!(call.algorithm, SignatureAlgorithm::RsaPss);
    assert_eq!(call.digest.len(), 64);
}

#[test]
fn a_reply_after_the_request_ended_sends_nothing() {
    let p = Cert::p256();
    let (mut h, tag, key) = signing(&p, SHA256, &["ECDSA"]);
    h.ui(websign_ui_model::confirm::UiEvent::Cancel {
        key,
        code: websign_protocol::ErrorCode::UserCancelled,
    });
    h.take();
    let out = h.signed_ok(tag, &p, SHA256, ECDSA);
    assert!(
        out.results().is_empty(),
        "the caller was already told it failed"
    );
    assert!(out.errors().is_empty());
}
