//! SPEC §8.4 and decision D11: a caller that is not remembered gets the
//! certificate only after the person presses Continue.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN, OTHER_ORIGIN};
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::UiEvent;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

fn started(h: &mut Harness, cert: &Cert) -> websign_ui_model::confirm::port::RequestKey {
    let out = h.begin("s1", ORIGIN, SHA256);
    let opens = out.opens();
    assert_eq!(opens.len(), 1);
    assert!(!opens[0].remembered);
    let key = opens[0].key;
    h.listed(&[cert]);
    key
}

#[test]
fn no_need_digest_until_continue() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;

    let out = h.listed(&[&p]);
    assert!(
        out.need_digests().is_empty(),
        "nothing is released after the listing"
    );
    assert!(out.frames.is_empty());
    assert!(out.certificates_count() >= 1, "the window shows the list");

    // Selecting is not consent either.
    let out = h.ui_out(UiEvent::Selected {
        key,
        fingerprint: p.fingerprint,
    });
    assert!(out.need_digests().is_empty());

    let out = h.ui_out(UiEvent::Continue {
        key,
        fingerprint: p.fingerprint,
    });
    let needs = out.need_digests();
    assert_eq!(needs.len(), 1);
    assert_eq!(needs[0].1.seq, 1);
    assert_eq!(needs[0].1.certificate.fingerprint.as_str(), p.hex());
    assert_eq!(out.digest_pending_count(), 1);
}

#[test]
fn cancel_before_continue_leaves_the_caller_with_an_error_and_no_certificate() {
    let p = Cert::person();
    let mut h = Harness::native_ready();
    let key = started(&mut h, &p);

    let out = h.ui_out(UiEvent::Cancel {
        key,
        code: ErrorCode::UserCancelled,
    });
    assert_eq!(
        out.only_error(),
        ("s1".to_owned(), ErrorCode::UserCancelled)
    );

    // D11: nothing about the holder may have reached the caller.
    let json = out.frames_json();
    assert!(out.need_digests().is_empty());
    assert!(!json.contains(&p.hex()), "no fingerprint");
    assert!(!json.contains("certificate"), "no certificate");
    assert!(!json.contains("displayName"), "no holder name");
    assert!(out.signs().is_empty());
    assert_eq!(
        out.finished().len(),
        1,
        "the window is told the request ended"
    );
}

#[test]
fn the_error_code_is_the_one_the_window_reported() {
    let p = Cert::p256();
    for code in [
        ErrorCode::NoCertificates,
        ErrorCode::UserCancelled,
        ErrorCode::PinLocked,
    ] {
        let mut h = Harness::native_ready();
        let key = started(&mut h, &p);
        let out = h.ui_out(UiEvent::Cancel { key, code });
        assert_eq!(out.only_error().1, code);
    }
}

#[test]
fn continue_for_a_certificate_that_was_not_listed_is_ignored() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    let key = started(&mut h, &p);
    let out = h.ui_out(UiEvent::Continue {
        key,
        fingerprint: r.fingerprint,
    });
    assert!(
        out.need_digests().is_empty(),
        "the caller cannot be handed an unlisted certificate"
    );
}

#[test]
fn a_site_remembered_elsewhere_is_still_new_here() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(OTHER_ORIGIN, &[&p]);
    h.begin("s1", ORIGIN, SHA256);
    let out = h.listed(&[&p]);
    assert!(out.need_digests().is_empty(), "consent is per origin");
}

#[test]
fn after_continue_the_flow_signs_like_any_other() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = started(&mut h, &p);
    h.ui(UiEvent::Continue {
        key,
        fingerprint: p.fingerprint,
    });
    h.take();
    assert_eq!(h.answer_digest("s1", 1, SHA256).digest_ready_count(), 1);
    let out = h.press_sign(key, &p, None);
    let tag = out.signs()[0].tag;
    let out = h.signed_ok(tag, &p, SHA256, SignatureAlgorithm::Ecdsa);
    assert_eq!(out.results().len(), 1);
}

#[test]
fn a_digest_the_page_volunteers_early_ends_the_request() {
    // A sequence number the app never issued is a broken client, not a late
    // answer; nothing about the certificate has left.
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    started(&mut h, &p);
    let out = h.answer_digest("s1", 1, SHA256);
    assert_eq!(
        out.only_error(),
        ("s1".to_owned(), ErrorCode::InvalidRequest)
    );
    assert!(out.need_digests().is_empty() && out.signs().is_empty());
    assert_eq!(out.digest_ready_count(), 0);
}

#[test]
fn a_new_desktop_program_is_not_remembered_either() {
    let p = Cert::p256();
    let mut h = Harness::desktop_ready();
    h.send(common::wire::sign_begin("s1", None, "SHA-256"));
    let out = h.take();
    assert!(!out.opens()[0].remembered);
    let out = h.listed(&[&p]);
    assert!(out.need_digests().is_empty());
}
