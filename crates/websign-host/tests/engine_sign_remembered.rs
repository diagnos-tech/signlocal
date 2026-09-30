//! SPEC §8.3: the whole happy path for a remembered site, and the rule that
//! every signature still needs the person.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN};
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_protocol::types::{HashName, SignatureAlgorithmName};
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::{Mode, RequestKey};

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

#[test]
fn remembered_site_runs_from_sign_begin_to_sign_result() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);

    // sign.begin: window opens, key store lists.
    let out = h.begin("s1", ORIGIN, SHA256);
    let opens = out.opens();
    assert_eq!(opens.len(), 1);
    assert!(
        opens[0].remembered,
        "the window knows the caller is remembered"
    );
    assert!(matches!(opens[0].mode, Mode::Sign { .. }));
    assert_eq!(opens[0].position, (1, 1));
    assert_eq!(
        out.lists(),
        [false],
        "first listing may come from the cache"
    );
    assert!(out.frames.is_empty(), "nothing is sent to the page yet");
    let key = opens[0].key;

    // Listed: certificates shown, preselected one released at once.
    let out = h.listed(&[&p]);
    assert!(out.certificates_count() >= 1);
    let needs = out.need_digests();
    assert_eq!(needs.len(), 1);
    let (id, need) = &needs[0];
    assert_eq!(id, "s1");
    assert_eq!(need.seq, 1);
    assert_eq!(need.hash, HashName::Sha256);
    assert_eq!(need.algorithm, SignatureAlgorithmName::Ecdsa);
    assert_eq!(need.certificate.fingerprint.as_str(), p.hex());
    assert_eq!(need.certificate.der.as_bytes(), p.der.as_slice());
    assert_eq!(out.digest_pending_count(), 1);

    // Digest: the window can show the code.
    let out = h.answer_digest("s1", 1, SHA256);
    assert_eq!(out.digest_ready_count(), 1);
    assert!(out.frames.is_empty());
    assert!(out.signs().is_empty(), "a digest alone never signs");

    // Sign: one key store call with exactly what the page sent.
    let out = h.press_sign(key, &p, None);
    let signs = out.signs();
    assert_eq!(signs.len(), 1);
    assert_eq!(signs[0].hash, SHA256);
    assert_eq!(signs[0].algorithm, SignatureAlgorithm::Ecdsa);
    assert_eq!(signs[0].digest, common::certs::digest(SHA256));
    assert_eq!(signs[0].key.fingerprint, p.fingerprint);
    assert_eq!(signs[0].key.path, 0);
    assert_eq!(signs[0].parent_window, Some(common::fakes::PARENT_WINDOW));
    assert_eq!(out.signing_count(), 1);

    // Signed and verified: one sign.result, window says Signed.
    let out = h.signed_ok(signs[0].tag, &p, SHA256, SignatureAlgorithm::Ecdsa);
    let results = out.results();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "s1");
    let result = &results[0].1;
    assert_eq!(result.certificate.fingerprint.as_str(), p.hex());
    assert_eq!(result.hash, HashName::Sha256);
    assert_eq!(result.algorithm, SignatureAlgorithmName::Ecdsa);
    assert_eq!(
        result.signature.as_bytes(),
        p.signature(SHA256, SignatureAlgorithm::Ecdsa).as_slice()
    );
    assert!(out.errors().is_empty());
    assert_eq!(out.finished().len(), 1);
    assert_eq!(out.finished()[0].0, key);
}

#[test]
fn an_unanswered_chain_leaves_the_result_without_one() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let out = h.press_sign(key, &p, None);
    let tag = out.signs()[0].tag;
    let out = h.signed_ok(tag, &p, SHA256, SignatureAlgorithm::Ecdsa);
    assert!(out.results()[0].1.certificate.chain.is_empty());
}

#[test]
fn the_chain_is_read_on_release_and_reaches_the_result() {
    let p = Cert::p256();
    let issuer = vec![0x30, 0x03, 0x02, 0x01, 0x07];
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    let listed = h.listed(&[&p]);
    assert_eq!(
        listed.key_kinds(),
        ["chain"],
        "asked when the certificate leaves"
    );
    assert!(
        listed.need_digests()[0].1.certificate.chain.is_empty(),
        "the digest request does not wait for it"
    );
    h.chains(&listed, std::slice::from_ref(&issuer));
    h.answer_digest("s1", 1, SHA256);
    let out = h.press_sign(key, &p, None);
    let tag = out.signs()[0].tag;
    let out = h.signed_ok(tag, &p, SHA256, SignatureAlgorithm::Ecdsa);
    let chain = &out.results()[0].1.certificate.chain;
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].as_bytes(), issuer.as_slice());
}

#[test]
fn every_signature_needs_its_own_confirmation() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let out = h.press_sign(key, &p, None);
    let tag = out.signs()[0].tag;
    assert_eq!(
        h.signed_ok(tag, &p, SHA256, SignatureAlgorithm::Ecdsa)
            .results()
            .len(),
        1
    );

    // A second request from the same remembered site: window again, and no
    // key store signature until the person presses Sign again.
    let out = h.begin("s2", ORIGIN, SHA256);
    assert_eq!(out.opens().len(), 1);
    let key2 = out.opens()[0].key;
    assert_ne!(key2, key, "a new request is a new window request");
    h.listed(&[&p]);
    let out = h.answer_digest("s2", 1, SHA256);
    assert_eq!(out.digest_ready_count(), 1);
    assert!(out.signs().is_empty());
    assert!(out.results().is_empty());
    assert_eq!(h.press_sign(key2, &p, None).signs().len(), 1);
}

#[test]
fn a_ui_sign_before_the_digest_arrived_does_nothing() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    assert_eq!(h.listed(&[&p]).need_digests().len(), 1);
    let out = h.press_sign(key, &p, None);
    assert!(out.signs().is_empty(), "still awaiting the digest");
    assert!(out.frames.is_empty());
}

#[test]
fn a_ui_sign_for_another_certificate_does_nothing() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let out = h.press_sign(key, &r, None);
    assert!(
        out.signs().is_empty(),
        "the ready digest belongs to the other certificate"
    );
}

#[test]
fn ui_events_for_an_unknown_request_key_are_ignored() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.drive_to_ready("s1", ORIGIN, &p);
    let out = h.press_sign(RequestKey(9_999_999), &p, None);
    assert!(out.signs().is_empty() && out.frames.is_empty());
}

#[test]
fn the_window_never_sees_the_digest_only_its_code() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.begin("s1", ORIGIN, SHA256);
    h.listed(&[&p]);
    let out = h.answer_digest("s1", 1, SHA256);
    let expected = websign_protocol::verification_code(&common::certs::digest(SHA256));
    let shown = out.ui.iter().find_map(|c| match c {
        UiCommand::DigestReady { code, .. } => Some(code.clone()),
        _ => None,
    });
    assert_eq!(shown, expected);
}
