//! SPEC §8.10: decision timeout (300 s on screen) and digest timeout (60 s per
//! `need_digest`), driven by a manual clock.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN};
use websign_core::HashAlgorithm;
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::Finish;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

#[test]
fn nobody_decides_within_300_seconds() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[&p]);

    h.pass(299);
    assert!(h.take().is_silent(), "still within the limit");

    h.pass(1);
    let out = h.take();
    assert_eq!(out.only_error(), ("s1".to_owned(), ErrorCode::Timeout));
    assert_eq!(out.finished(), [(key, Finish::Timeout)]);
    assert!(out.need_digests().is_empty(), "D11 holds through a timeout");
}

#[test]
fn the_window_reports_the_countdown_time_it_was_given() {
    let mut h = Harness::native_ready();
    let out = h.begin("s1", ORIGIN, SHA256);
    assert_eq!(out.opens()[0].timeout_secs, 300);
}

#[test]
fn the_caller_does_not_answer_need_digest_within_60_seconds() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    assert_eq!(h.listed(&[&p]).need_digests().len(), 1);

    h.pass(59);
    assert!(h.take().is_silent());

    h.pass(1);
    let out = h.take();
    assert_eq!(out.only_error(), ("s1".to_owned(), ErrorCode::Timeout));
    assert_eq!(
        out.finished(),
        [(key, Finish::DigestTimeout)],
        "the window blames the site, not the person"
    );
}

#[test]
fn the_digest_clock_restarts_with_each_need_digest() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p, &r]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[&p, &r]);

    h.pass(50);
    h.take();
    h.ui(UiEvent::Selected {
        key,
        fingerprint: r.fingerprint,
    }); // seq 2 at t = 50
    h.take();

    h.pass(59); // t = 109: 59 s after the second need_digest
    assert!(h.take().is_silent(), "the first deadline no longer applies");
    h.pass(1);
    assert_eq!(h.take().only_error().1, ErrorCode::Timeout);
}

#[test]
fn an_answered_digest_stops_the_digest_clock() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.drive_to_ready("s1", ORIGIN, &p);
    h.pass(120);
    assert!(
        h.take().is_silent(),
        "the person still has the decision limit"
    );
}

#[test]
fn a_late_answer_after_a_timeout_is_invalid() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.begin("s1", ORIGIN, SHA256);
    h.listed(&[&p]);
    h.pass(60);
    h.take();
    let out = h.answer_digest("s1", 1, SHA256);
    assert_eq!(out.only_error().1, ErrorCode::InvalidRequest);
}

#[test]
fn the_decision_limit_starts_when_the_request_reaches_the_screen() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let first = h.begin("a", ORIGIN, SHA256).opens()[0].key;
    h.begin("b", ORIGIN, SHA256); // waits
    h.listed(&[&p]);

    h.pass(200);
    h.take();
    h.ui(UiEvent::Cancel {
        key: first,
        code: ErrorCode::UserCancelled,
    });
    let out = h.take();
    assert_eq!(out.opens().len(), 1, "b takes the screen at t = 200");

    h.pass(299);
    assert!(
        h.take().errors().is_empty(),
        "b has its own 300 s from the moment it was shown"
    );
    h.pass(1);
    assert_eq!(h.take().only_error(), ("b".to_owned(), ErrorCode::Timeout));
}

#[test]
fn ticks_with_nothing_open_do_nothing() {
    let mut h = Harness::native_ready();
    h.pass(10_000);
    assert!(h.take().frames.is_empty());
}

#[test]
fn a_signing_request_does_not_time_out_while_the_key_store_works() {
    // An OS PIN dialog or a slow token may take minutes; the person has
    // decided, so the decision limit is suspended while the key store signs.
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let tag = h.press_sign(key, &p, None).signs()[0].tag;
    h.pass(600);
    assert!(h.take().is_silent());
    let out = h.signed_ok(tag, &p, SHA256, websign_core::SignatureAlgorithm::Ecdsa);
    assert_eq!(out.results().len(), 1);
}

#[test]
fn a_failed_signing_after_the_limit_ends_at_the_next_tick() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let tag = h.press_sign(key, &p, None).signs()[0].tag;
    h.pass(600);
    h.signed_err(tag, websign_keystores::KeystoreError::WrongPin);
    assert_eq!(h.tick(), websign_host::Control::Continue);
    let out = h.take();
    assert_eq!(out.only_error(), ("s1".to_owned(), ErrorCode::Timeout));
    assert_eq!(out.finished(), [(key, Finish::Timeout)]);
}
