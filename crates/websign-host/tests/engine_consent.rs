//! Consent per caller (ux.md §4.10, SPEC §4 and §7): remembered, ticked,
//! revocable, and never for callers that cannot be remembered.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN, OTHER_ORIGIN, wire};
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::RequestKey;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;
const ECDSA: SignatureAlgorithm = SignatureAlgorithm::Ecdsa;

fn status_remembered(h: &mut Harness, origin: &str) -> bool {
    h.send(wire::status("st", Some(origin)));
    let out = h.take();
    match &out.frames[0].message {
        websign_protocol::AppMessage::Status(reply) => reply.remembered,
        other => panic!("expected status, got {other:?}"),
    }
}

fn sign_with_remember(h: &mut Harness, origin: &str, cert: &Cert, remember: bool) -> RequestKey {
    let key = h.begin("s1", origin, SHA256).opens()[0].key;
    h.listed(&[cert]);
    h.ui(UiEvent::Continue {
        key,
        fingerprint: cert.fingerprint,
    });
    h.take();
    h.answer_digest("s1", 1, SHA256);
    h.ui(UiEvent::Sign {
        key,
        fingerprint: cert.fingerprint,
        via: 0,
        pin: None,
        remember,
    });
    let tag = h.take().signs()[0].tag;
    let out = h.signed_ok(tag, cert, SHA256, ECDSA);
    assert_eq!(out.results().len(), 1);
    key
}

#[test]
fn status_reports_whether_the_caller_is_remembered() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    assert!(!status_remembered(&mut h, ORIGIN));
    h.remember(ORIGIN, &[&p]);
    assert!(status_remembered(&mut h, ORIGIN));
    assert!(
        !status_remembered(&mut h, OTHER_ORIGIN),
        "consent is per origin"
    );
}

#[test]
fn status_never_opens_a_window_or_touches_the_key_store() {
    let mut h = Harness::native_ready();
    h.send(wire::status("st", Some(ORIGIN)));
    let out = h.take();
    assert!(out.ui.is_empty() && out.keys.is_empty());
}

#[test]
fn revoking_consent_takes_effect_on_the_next_request() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    assert!(status_remembered(&mut h, ORIGIN));

    h.state.borrow_mut().consent.clear(); // Diagnostics › Allowed sites › Revoke

    assert!(!status_remembered(&mut h, ORIGIN));
    let out = h.begin("s1", ORIGIN, SHA256);
    assert!(!out.opens()[0].remembered);
    assert!(
        h.listed(&[&p]).need_digests().is_empty(),
        "D11 applies again"
    );
}

#[test]
fn consent_is_read_at_each_request_not_once_per_connection() {
    // Several host processes share the file (store/mod.rs): a change made by
    // another process must be seen by this one.
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    assert!(!h.begin("a", ORIGIN, SHA256).opens()[0].remembered);
    h.send(wire::cancel("a"));
    h.take();
    h.remember(ORIGIN, &[&p]);
    assert!(h.begin("b", ORIGIN, SHA256).opens()[0].remembered);
}

#[test]
fn ticking_remember_on_a_signature_stores_consent_and_records_use() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    sign_with_remember(&mut h, ORIGIN, &p, true);
    let state = h.state.borrow();
    assert_eq!(state.consent.len(), 1);
    assert_eq!(state.consent[0].key, ORIGIN);
    assert_eq!(state.consent[0].certificates, [p.hex()]);
    assert_eq!(state.usage.first(), Some(&p.hex()));
}

#[test]
fn not_ticking_remember_stores_no_consent_but_still_records_certificate_use() {
    // SPEC §7: usage is anonymous (fingerprints by recency, no caller), so
    // every successful answer records it.
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    sign_with_remember(&mut h, ORIGIN, &p, false);
    let state = h.state.borrow();
    assert!(state.consent.is_empty());
    assert_eq!(state.usage.first(), Some(&p.hex()));
}

#[test]
fn a_remembered_caller_moves_the_certificate_it_used_to_the_front() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p, &r]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[&p, &r]);
    h.ui(UiEvent::Selected {
        key,
        fingerprint: r.fingerprint,
    });
    h.take();
    h.answer_digest("s1", 2, SHA256);
    let tag = h.press_sign(key, &r, None).signs()[0].tag;
    h.signed_ok(tag, &r, SHA256, SignatureAlgorithm::RsaPkcs1v15);
    assert_eq!(h.state.borrow().consent[0].certificates[0], r.hex());
}

#[test]
fn a_failed_signature_records_no_use() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let tag = h.press_sign(key, &p, None).signs()[0].tag;
    h.keys(websign_host::ports::KeyReply::Signed {
        tag,
        result: Ok(vec![0; 64]),
    });
    h.take();
    assert!(h.state.borrow().usage.is_empty());
}

#[test]
fn an_ip_origin_cannot_be_remembered_even_if_the_event_says_so() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let out = h.begin("s1", "https://192.0.2.10", SHA256);
    let open = &out.opens()[0];
    assert!(!open.can_remember, "the window must not offer the checkbox");
    let key = open.key;
    h.listed(&[&p]);
    h.ui(UiEvent::Continue {
        key,
        fingerprint: p.fingerprint,
    });
    h.take();
    h.answer_digest("s1", 1, SHA256);
    h.ui(UiEvent::Sign {
        key,
        fingerprint: p.fingerprint,
        via: 0,
        pin: None,
        remember: true,
    });
    let tag = h.take().signs()[0].tag;
    h.signed_ok(tag, &p, SHA256, ECDSA);
    assert!(h.state.borrow().consent.is_empty());
}

#[test]
fn a_desktop_program_can_be_remembered_and_is_then_released_at_once() {
    let p = Cert::p256();
    let mut h = Harness::desktop_ready();
    h.send(wire::sign_begin("s1", None, "SHA-256"));
    let key = h.take().opens()[0].key;
    h.listed(&[&p]);
    h.ui(UiEvent::Continue {
        key,
        fingerprint: p.fingerprint,
    });
    h.take();
    h.answer_digest("s1", 1, SHA256);
    h.ui(UiEvent::Sign {
        key,
        fingerprint: p.fingerprint,
        via: 0,
        pin: None,
        remember: true,
    });
    let tag = h.take().signs()[0].tag;
    h.signed_ok(tag, &p, SHA256, ECDSA);

    h.send(wire::sign_begin("s2", None, "SHA-256"));
    let out = h.take();
    assert!(out.opens()[0].remembered);
    assert_eq!(h.listed(&[&p]).need_digests().len(), 1);
}

#[test]
fn the_window_is_told_which_caller_asks_from_the_transport_never_the_payload() {
    use websign_ui_model::confirm::port::CallerView;
    let mut h = Harness::native_ready();
    let out = h.begin("s1", ORIGIN, SHA256);
    match &out.opens()[0].caller {
        CallerView::Web {
            origin,
            top,
            browser,
        } => {
            assert_eq!(origin.canonical, ORIGIN);
            assert!(top.is_none());
            assert_eq!(*browser, websign_protocol::types::BrowserName::Chrome);
        }
        other => panic!("expected a web caller, got {other:?}"),
    }

    let mut d = Harness::desktop_ready();
    d.send(wire::sign_begin("s1", None, "SHA-256"));
    let out = d.take();
    assert!(matches!(out.opens()[0].caller, CallerView::Desktop { .. }));
}

#[test]
fn a_frame_from_another_origin_shows_the_tab_origin_as_top() {
    use websign_ui_model::confirm::port::CallerView;
    let mut h = Harness::native_ready();
    let web = wire::framed(OTHER_ORIGIN, ORIGIN);
    h.send(wire::sign_begin_raw_web("s1", web, "SHA-256"));
    let out = h.take();
    match &out.opens()[0].caller {
        CallerView::Web { origin, top, .. } => {
            assert_eq!(origin.canonical, OTHER_ORIGIN);
            assert_eq!(top.as_ref().map(|t| t.canonical.as_str()), Some(ORIGIN));
        }
        other => panic!("expected a web caller, got {other:?}"),
    }
}
