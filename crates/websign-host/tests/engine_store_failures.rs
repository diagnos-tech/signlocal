//! SPEC §7: "store failures never fail a request: the engine logs and
//! continues without persistence."

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN, wire};
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_ui_model::confirm::UiEvent;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

#[test]
fn hello_works_when_the_connection_store_fails() {
    let mut h = Harness::native();
    h.state.borrow_mut().failing = true;
    h.send(wire::hello_native("h", 1, 1));
    assert_eq!(h.take().kinds(), ["hello"]);
}

#[test]
fn an_unreadable_consent_store_means_a_new_caller_and_status_still_answers() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.state.borrow_mut().failing = true;

    h.send(wire::status("st", Some(ORIGIN)));
    let out = h.take();
    assert_eq!(out.kinds(), ["status"]);

    let out = h.begin("s1", ORIGIN, SHA256);
    assert!(!out.opens()[0].remembered, "when in doubt, ask (D11)");
    assert!(h.listed(&[&p]).need_digests().is_empty());
}

#[test]
fn a_signature_completes_even_if_nothing_can_be_saved() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[&p]);
    h.ui(UiEvent::Continue {
        key,
        fingerprint: p.fingerprint,
    });
    h.take();
    h.answer_digest("s1", 1, SHA256);

    h.state.borrow_mut().failing = true;
    h.ui(UiEvent::Sign {
        key,
        fingerprint: p.fingerprint,
        via: 0,
        pin: None,
        remember: true,
    });
    let tag = h.take().signs()[0].tag;
    let out = h.signed_ok(tag, &p, SHA256, SignatureAlgorithm::Ecdsa);
    assert_eq!(
        out.results().len(),
        1,
        "the person's signature is not lost to a disk error"
    );
    assert!(out.errors().is_empty());
}

#[test]
fn a_choose_completes_even_if_consent_cannot_be_saved() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.send(wire::choose("c1", Some(ORIGIN)));
    let key = h.take().opens()[0].key;
    h.listed(&[&p]);
    h.state.borrow_mut().failing = true;
    let asked = h.ui_out(UiEvent::Choose {
        key,
        fingerprint: p.fingerprint,
        remember: true,
    });
    let out = h.chains(&asked, &[]);
    assert_eq!(out.kinds(), ["choose.result"]);
}

#[test]
fn a_failing_error_store_does_not_hide_the_failure_from_the_window() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let tag = h.press_sign(key, &p, None).signs()[0].tag;
    h.state.borrow_mut().failing = true;
    let out = h.signed_err(
        tag,
        websign_keystores::KeystoreError::Other("boom".to_owned()),
    );
    assert_eq!(out.failures().len(), 1);
}
