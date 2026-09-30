//! SPEC §8.5 and §4.2: switching certificate, stale digests, and the
//! algorithm the app chooses.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN};
use serde_json::json;
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_protocol::types::SignatureAlgorithmName;
use websign_ui_model::confirm::UiEvent;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

#[test]
fn switching_certificate_asks_again_and_ignores_the_late_digest() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    let out = h.listed(&[&p, &r]);
    assert_eq!(out.need_digests()[0].1.seq, 1);
    assert_eq!(
        out.need_digests()[0].1.certificate.fingerprint.as_str(),
        p.hex()
    );

    // The person picks the RSA certificate: a second question, seq 2.
    let out = h.ui_out(UiEvent::Selected {
        key,
        fingerprint: r.fingerprint,
    });
    let needs = out.need_digests();
    assert_eq!(needs.len(), 1);
    assert_eq!(needs[0].1.seq, 2);
    assert_eq!(needs[0].1.certificate.fingerprint.as_str(), r.hex());
    assert_eq!(needs[0].1.algorithm, SignatureAlgorithmName::RsaPkcs1v15);
    assert_eq!(out.digest_pending_count(), 1);

    // The digest for seq 1 was already in flight: ignored without a trace.
    let out = h.answer_digest("s1", 1, SHA256);
    assert!(
        out.is_silent(),
        "a stale digest changes nothing: {:?}",
        out.kinds()
    );

    // The digest for seq 2 is the one that counts.
    let out = h.answer_digest("s1", 2, SHA256);
    assert_eq!(out.digest_ready_count(), 1);
    assert!(out.errors().is_empty());
}

#[test]
fn a_new_site_switching_certificate_still_waits_for_continue() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[&p, &r]);

    let out = h.ui_out(UiEvent::Selected {
        key,
        fingerprint: r.fingerprint,
    });
    assert!(out.need_digests().is_empty());

    let out = h.ui_out(UiEvent::Continue {
        key,
        fingerprint: r.fingerprint,
    });
    let needs = out.need_digests();
    assert_eq!(needs.len(), 1);
    assert_eq!(needs[0].1.certificate.fingerprint.as_str(), r.hex());
}

#[test]
fn switching_after_ready_goes_back_to_waiting_for_a_digest() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[&p, &r]);
    assert_eq!(h.answer_digest("s1", 1, SHA256).digest_ready_count(), 1);

    let out = h.ui_out(UiEvent::Selected {
        key,
        fingerprint: r.fingerprint,
    });
    assert_eq!(out.need_digests()[0].1.seq, 2);

    // Ready again only with the new digest; Sign for either is refused meanwhile.
    assert!(h.press_sign(key, &r, None).signs().is_empty());
    assert!(h.press_sign(key, &p, None).signs().is_empty());
    assert_eq!(h.answer_digest("s1", 2, SHA256).digest_ready_count(), 1);
    assert_eq!(h.press_sign(key, &r, None).signs().len(), 1);
}

#[test]
fn a_remembered_caller_gets_the_certificate_it_used_last_preselected() {
    // SPEC §4: the engine's list context makes the certificate last used
    // here win over listing order ([p, r]).
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&r]);
    h.begin("s1", ORIGIN, SHA256);
    let out = h.listed(&[&p, &r]);
    assert_eq!(
        out.need_digests()[0].1.certificate.fingerprint.as_str(),
        r.hex()
    );
}

#[test]
fn the_certificate_named_in_sign_begin_is_preselected() {
    // SPEC §4: `sign.begin.certificate` wins over the last used one; it is
    // released at once only because the caller is remembered.
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let frame = common::wire::sign_begin_with(
        "s1",
        ORIGIN,
        json!({ "hash": "SHA-256", "certificate": r.hex() }),
    );
    h.send(frame);
    h.take();
    let out = h.listed(&[&p, &r]);
    assert_eq!(
        out.need_digests()[0].1.certificate.fingerprint.as_str(),
        r.hex()
    );
}

#[test]
fn a_requested_certificate_that_is_not_there_is_ignored() {
    let p = Cert::p256();
    let missing = Cert::rsa();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let frame =
        common::wire::sign_begin_with("s1", ORIGIN, json!({ "certificate": missing.hex() }));
    h.send(frame);
    h.take();
    let out = h.listed(&[&p]);
    assert!(out.errors().is_empty());
    assert_eq!(
        out.need_digests()[0].1.certificate.fingerprint.as_str(),
        p.hex()
    );
}

#[test]
fn the_algorithm_is_the_first_requested_one_the_key_supports() {
    let r = Cert::rsa();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&r]);
    let frame = common::wire::sign_begin_with(
        "s1",
        ORIGIN,
        json!({ "algorithms": ["ECDSA", "RSASSA-PSS", "RSASSA-PKCS1-v1_5"] }),
    );
    h.send(frame);
    let key = h.take().opens()[0].key;
    let out = h.listed(&[&r]);
    assert_eq!(
        out.need_digests()[0].1.algorithm,
        SignatureAlgorithmName::RsaPss
    );
    h.answer_digest("s1", 1, SHA256);
    let signs = h.press_sign(key, &r, None).signs();
    assert_eq!(signs[0].algorithm, SignatureAlgorithm::RsaPss);
}

#[test]
fn without_a_preference_ecdsa_is_tried_first_then_pkcs1() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.begin("s1", ORIGIN, SHA256);
    let out = h.listed(&[&p, &r]);
    assert_eq!(
        out.need_digests()[0].1.algorithm,
        SignatureAlgorithmName::Ecdsa
    );
}

#[test]
fn a_certificate_that_supports_no_requested_algorithm_is_never_released() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let frame =
        common::wire::sign_begin_with("s1", ORIGIN, json!({ "algorithms": ["RSASSA-PSS"] }));
    h.send(frame);
    h.take();
    let out = h.listed(&[&p]);
    assert!(
        out.need_digests().is_empty(),
        "disabled rows cannot be selected"
    );
    assert!(
        out.errors().is_empty(),
        "the window stays open for another certificate"
    );
}
