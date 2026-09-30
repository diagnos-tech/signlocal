//! SPEC §8.20 and D11: a remembered caller gets only the certificates its
//! consent covers; every other certificate waits for Continue. Also §8.21:
//! interpreters and shells are never remembered.

mod common;

use std::path::PathBuf;

use common::harness::Harness;
use common::{Cert, ORIGIN, wire};
use websign_core::present::caller::DesktopCaller;
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_host::session::Transport;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::RequestKey;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

/// Continue, digest and Sign for `cert` on the request on screen.
fn continue_and_sign(h: &mut Harness, id: &str, key: RequestKey, cert: &Cert, remember: bool) {
    let out = h.ui_out(UiEvent::Continue {
        key,
        fingerprint: cert.fingerprint,
    });
    let seq = out.need_digests()[0].1.seq;
    h.answer_digest(id, seq, SHA256);
    h.ui(UiEvent::Sign {
        key,
        fingerprint: cert.fingerprint,
        via: 0,
        pin: None,
        remember,
    });
    let tag = h.take().signs()[0].tag;
    let algorithm = if cert.fingerprint == Cert::p256().fingerprint {
        SignatureAlgorithm::Ecdsa
    } else {
        SignatureAlgorithm::RsaPkcs1v15
    };
    assert_eq!(h.signed_ok(tag, cert, SHA256, algorithm).results().len(), 1);
}

#[test]
fn a_remembered_site_gets_nothing_from_someone_elses_token_before_continue() {
    // Dr A remembered the site with her token; Dr B's token is the only one
    // plugged in on the shared computer.
    let (mine, theirs) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&mine]);
    let open = h.begin("s1", ORIGIN, SHA256).opens()[0].clone();
    assert!(open.remembered);
    assert_eq!(open.consented, [mine.fingerprint]);

    let out = h.listed(&[&theirs]);
    assert!(
        out.need_digests().is_empty(),
        "nothing released: {:?}",
        out.kinds()
    );
    assert!(out.chain_tags().is_empty());
    assert_eq!(out.digest_pending_count(), 0);

    let out = h.ui_out(UiEvent::Continue {
        key: open.key,
        fingerprint: theirs.fingerprint,
    });
    let needs = out.need_digests();
    assert_eq!(needs.len(), 1, "released only after Continue");
    assert_eq!(needs[0].1.certificate.fingerprint.as_str(), theirs.hex());
}

#[test]
fn arrowing_through_the_list_releases_nothing_new() {
    let (mine, b, c) = (Cert::p256(), Cert::rsa(), Cert::rsa_b());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&mine]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    let out = h.listed(&[&mine, &b, &c]);
    assert_eq!(out.need_digests().len(), 1, "the consented one, at once");

    for cert in [&b, &c, &b] {
        let out = h.ui_out(UiEvent::Selected {
            key,
            fingerprint: cert.fingerprint,
        });
        assert!(out.need_digests().is_empty(), "moving to {}", cert.hex());
        assert!(out.chain_tags().is_empty());
    }
    let out = h.ui_out(UiEvent::Selected {
        key,
        fingerprint: mine.fingerprint,
    });
    let released: Vec<String> = out
        .need_digests()
        .iter()
        .map(|(_, need)| need.certificate.fingerprint.as_str().to_owned())
        .collect();
    assert_eq!(released, [mine.hex()], "only the consented one again");
}

#[test]
fn the_remembered_certificate_is_still_released_at_once() {
    let (mine, other) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&mine]);
    h.begin("s1", ORIGIN, SHA256);
    let out = h.listed(&[&other, &mine]);
    let needs = out.need_digests();
    assert_eq!(needs.len(), 1);
    assert_eq!(needs[0].1.certificate.fingerprint.as_str(), mine.hex());
}

#[test]
fn after_revoke_every_certificate_waits_for_continue_again() {
    let mine = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&mine]);
    h.begin("s1", ORIGIN, SHA256);
    assert_eq!(h.listed(&[&mine]).need_digests().len(), 1);
    h.send(wire::cancel("s1"));
    h.take();

    h.state.borrow_mut().consent.clear(); // Diagnostics › Allowed sites › Revoke

    let open = h.begin("s2", ORIGIN, SHA256).opens()[0].clone();
    assert!(!open.remembered && open.consented.is_empty());
    assert!(h.listed(&[&mine]).need_digests().is_empty(), "back to D11");
}

#[test]
fn using_another_certificate_without_remember_does_not_extend_the_consent() {
    let (mine, other) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&mine]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[&other]);
    continue_and_sign(&mut h, "s1", key, &other, false);
    assert_eq!(h.state.borrow().consent[0].certificates, [mine.hex()]);

    h.begin("s2", ORIGIN, SHA256);
    assert!(h.listed(&[&other]).need_digests().is_empty());
}

#[test]
fn ticking_remember_adds_only_the_chosen_certificate() {
    let (mine, other, third) = (Cert::p256(), Cert::rsa(), Cert::rsa_b());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&mine]);
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[&other, &third]);
    continue_and_sign(&mut h, "s1", key, &other, true);
    assert_eq!(
        h.state.borrow().consent[0].certificates,
        [other.hex(), mine.hex()]
    );
}

fn interpreter() -> Harness {
    let caller = DesktopCaller {
        executable: PathBuf::from("/usr/bin/node"),
        product_name: None,
        signer: None,
    };
    let mut h = Harness::new(Transport::Desktop { caller }, (1, 1));
    h.send(wire::hello_desktop("h", 1, 1));
    h.take();
    h
}

#[test]
fn an_interpreter_cannot_be_remembered_even_when_remember_is_ticked() {
    let p = Cert::p256();
    let mut h = interpreter();
    h.send(wire::sign_begin("s1", None, "SHA-256"));
    let open = h.take().opens()[0].clone();
    assert!(!open.can_remember);
    h.listed(&[&p]);
    continue_and_sign(&mut h, "s1", open.key, &p, true);
    assert!(h.state.borrow().consent.is_empty());
}

#[test]
fn consent_stored_for_an_interpreter_by_an_older_version_is_ignored() {
    let p = Cert::p256();
    let mut h = interpreter();
    h.remember("path:/usr/bin/node", &[&p]);
    h.send(wire::sign_begin("s1", None, "SHA-256"));
    let open = h.take().opens()[0].clone();
    assert!(!open.remembered);
    assert!(h.listed(&[&p]).need_digests().is_empty());
}
