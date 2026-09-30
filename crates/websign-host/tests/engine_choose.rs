//! SPEC §8.12 and §5: `choose` for remembered and new callers (D2: never the
//! machine's list), with the issuer chain in every answer.

mod common;

use common::harness::{Harness, consent_key_of};
use common::{Cert, ORIGIN, wire};
use websign_host::ports::KeyReply;
use websign_protocol::{AppMessage, ErrorCode};
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::{Finish, Mode};

fn choose_certs(out: &common::Out) -> Vec<String> {
    out.frames
        .iter()
        .find_map(|f| match &f.message {
            AppMessage::ChooseResult(r) => Some(
                r.certificates
                    .iter()
                    .map(|c| c.fingerprint.as_str().to_owned())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

/// A new caller's `choose` on screen with `certs` listed; its window key.
fn on_screen(
    h: &mut Harness,
    id: &str,
    certs: &[&Cert],
) -> websign_ui_model::confirm::port::RequestKey {
    h.send(wire::choose(id, Some(ORIGIN)));
    let key = h.take().opens()[0].key;
    h.listed(certs);
    key
}

#[test]
fn a_remembered_caller_is_answered_without_a_window() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);

    h.send(wire::choose("c1", Some(ORIGIN)));
    let out = h.take();
    assert!(out.opens().is_empty(), "no window");
    assert_eq!(out.key_kinds(), ["list"]);

    let listed = h.listed(&[&r, &p]);
    assert!(listed.frames.is_empty(), "the chain comes first");
    assert_eq!(listed.key_kinds(), ["chain"]);
    let out = h.chains(&listed, &[]);
    assert_eq!(out.kinds(), ["choose.result"]);
    assert_eq!(out.frames[0].id.as_str(), "c1");
    assert_eq!(
        choose_certs(&out),
        [p.hex()],
        "only what the caller used before, never the list"
    );
    assert!(out.ui.is_empty() && listed.ui.is_empty());
}

#[test]
fn the_result_carries_the_certificate_and_its_chain_as_the_wire_defines_them() {
    let p = Cert::p256();
    let issuer = vec![0x30, 0x03, 0x02, 0x01, 0x07];
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.send(wire::choose("c1", Some(ORIGIN)));
    h.take();
    let listed = h.listed(&[&p]);
    let out = h.chains(&listed, std::slice::from_ref(&issuer));
    match &out.frames[0].message {
        AppMessage::ChooseResult(result) => {
            let c = &result.certificates[0];
            assert_eq!(c.der.as_bytes(), p.der.as_slice());
            assert_eq!(c.fingerprint.as_str(), p.hex());
            assert!(!c.display_name.is_empty());
            assert_eq!(c.not_before, p.info.not_before);
            assert_eq!(c.not_after, p.info.not_after);
            assert_eq!(c.chain.len(), 1);
            assert_eq!(c.chain[0].as_bytes(), issuer.as_slice());
        }
        other => panic!("expected choose.result, got {other:?}"),
    }
}

#[test]
fn the_answer_waits_for_every_chain() {
    let (p, b) = (Cert::p256(), Cert::rsa_b());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&b, &p]);
    h.send(wire::choose("c1", Some(ORIGIN)));
    h.take();
    let tags = h.listed(&[&p, &b]).chain_tags();
    assert_eq!(tags.len(), 2);
    h.keys(KeyReply::Chain {
        tag: tags[1],
        chain: Vec::new(),
    });
    assert!(h.take().frames.is_empty(), "one chain is still missing");
    h.keys(KeyReply::Chain {
        tag: tags[0],
        chain: Vec::new(),
    });
    assert_eq!(h.take().kinds(), ["choose.result"]);
}

#[test]
fn remembered_certificates_come_most_recent_first_and_only_those_present() {
    let (p, r, b) = (Cert::p256(), Cert::rsa(), Cert::rsa_b());
    let mut h = Harness::native_ready();
    // Record order is most recent first: b, then p, then r (r is unplugged).
    h.remember(ORIGIN, &[&b, &p, &r]);
    h.send(wire::choose("c1", Some(ORIGIN)));
    h.take();
    let out = h.listed_with_chains(&[&p, &b]);
    assert_eq!(choose_certs(&out), [b.hex(), p.hex()]);
}

#[test]
fn a_remembered_caller_whose_certificates_are_all_gone_gets_the_window() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.send(wire::choose("c1", Some(ORIGIN)));
    h.take();
    let out = h.listed(&[&r]);
    assert!(out.frames.is_empty(), "nothing is sent yet");
    let opens = out.opens();
    assert_eq!(opens.len(), 1, "the window took over");
    assert!(matches!(opens[0].mode, Mode::Choose));
    assert!(!opens[0].remembered);
    assert_eq!(out.key_kinds(), ["list"], "a listing for the window");
}

#[test]
fn revoking_a_remembered_caller_brings_the_window_back() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.send(wire::choose("c1", Some(ORIGIN)));
    assert!(h.take().opens().is_empty());
    assert_eq!(choose_certs(&h.listed_with_chains(&[&p])), [p.hex()]);

    // Diagnostics › Allowed sites › Remove, in another process.
    h.state
        .borrow_mut()
        .consent
        .retain(|r| r.key != consent_key_of(ORIGIN));

    h.send(wire::choose("c2", Some(ORIGIN)));
    let out = h.take();
    let opens = out.opens();
    assert_eq!(opens.len(), 1, "consent is read fresh for every request");
    assert!(matches!(opens[0].mode, Mode::Choose));
    assert!(!opens[0].remembered);
    let out = h.listed(&[&p]);
    assert!(
        out.frames.is_empty(),
        "D2: nothing before the person chooses"
    );
}

#[test]
fn a_new_caller_chooses_in_the_window_and_gets_exactly_one_certificate() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.send(wire::choose("c1", Some(ORIGIN)));
    let out = h.take();
    let key = out.opens()[0].key;
    assert!(matches!(out.opens()[0].mode, Mode::Choose));

    let out = h.listed(&[&p, &r]);
    assert!(
        out.frames.is_empty(),
        "D2: nothing before the person chooses"
    );
    assert_eq!(out.certificates_count(), 1);

    let asked = h.ui_out(UiEvent::Choose {
        key,
        fingerprint: r.fingerprint,
        remember: false,
    });
    assert_eq!(asked.key_kinds(), ["chain"]);
    assert!(asked.frames.is_empty());
    let out = h.chains(&asked, &[]);
    assert_eq!(choose_certs(&out), [r.hex()]);
    assert_eq!(out.finished(), [(key, Finish::Chosen)]);
    assert!(out.errors().is_empty());
    assert!(
        h.state.borrow().consent.is_empty(),
        "not remembered unless ticked"
    );
}

#[test]
fn ticking_remember_stores_consent_for_the_next_time() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = on_screen(&mut h, "c1", &[&p]);
    let asked = h.ui_out(UiEvent::Choose {
        key,
        fingerprint: p.fingerprint,
        remember: true,
    });
    h.chains(&asked, &[]);

    {
        let state = h.state.borrow();
        assert_eq!(state.consent.len(), 1);
        assert_eq!(state.consent[0].key, ORIGIN);
        assert_eq!(state.consent[0].certificates, [p.hex()]);
    }

    h.send(wire::choose("c2", Some(ORIGIN)));
    let out = h.take();
    assert!(
        out.opens().is_empty(),
        "now it is answered without a window"
    );
    let out = h.listed_with_chains(&[&p]);
    assert_eq!(choose_certs(&out), [p.hex()]);
}

#[test]
fn a_second_choose_press_while_the_chain_is_read_is_ignored() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    let key = on_screen(&mut h, "c1", &[&p, &r]);
    let asked = h.ui_out(UiEvent::Choose {
        key,
        fingerprint: p.fingerprint,
        remember: false,
    });
    let again = h.ui_out(UiEvent::Choose {
        key,
        fingerprint: r.fingerprint,
        remember: false,
    });
    assert!(again.is_silent());
    assert_eq!(choose_certs(&h.chains(&asked, &[])), [p.hex()]);
}

#[test]
fn choose_by_the_person_of_an_unlisted_certificate_is_ignored() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    let key = on_screen(&mut h, "c1", &[&p]);
    let out = h.ui_out(UiEvent::Choose {
        key,
        fingerprint: r.fingerprint,
        remember: false,
    });
    assert!(out.is_silent());
}

#[test]
fn closing_the_choose_window_reports_the_windows_code() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = on_screen(&mut h, "c1", &[&p]);
    let out = h.ui_out(UiEvent::Cancel {
        key,
        code: ErrorCode::UserCancelled,
    });
    assert_eq!(
        out.only_error(),
        ("c1".to_owned(), ErrorCode::UserCancelled)
    );
    assert_eq!(out.finished(), [(key, Finish::Aborted)]);
}

#[test]
fn choose_can_be_cancelled_by_the_client_and_times_out() {
    let mut h = Harness::native_ready();
    h.send(wire::choose("c1", Some(ORIGIN)));
    h.take();
    h.send(wire::cancel("c1"));
    assert_eq!(h.take().only_error().1, ErrorCode::Aborted);

    h.send(wire::choose("c2", Some(ORIGIN)));
    h.take();
    h.pass(299);
    assert!(h.take().is_silent());
    h.pass(1);
    assert_eq!(h.take().only_error(), ("c2".to_owned(), ErrorCode::Timeout));
}

#[test]
fn a_choose_filter_disables_what_it_does_not_accept() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.send(wire::choose_filtered("c1", ORIGIN, &["RSASSA-PSS"]));
    let key = h.take().opens()[0].key;
    h.listed(&[&p]);
    let out = h.ui_out(UiEvent::Choose {
        key,
        fingerprint: p.fingerprint,
        remember: false,
    });
    assert!(out.is_silent(), "a disabled row cannot be chosen");
}

#[test]
fn a_listing_with_no_certificates_leaves_the_choose_window_open() {
    let mut h = Harness::native_ready();
    h.send(wire::choose("c1", Some(ORIGIN)));
    h.take();
    let out = h.listed(&[]);
    assert!(
        out.frames.is_empty(),
        "the window shows its empty state and the person decides"
    );
    assert_eq!(out.certificates_count(), 1);
}

#[test]
fn a_desktop_program_is_remembered_by_its_path() {
    let p = Cert::p256();
    let mut h = Harness::desktop_ready();
    h.send(wire::choose("c1", None));
    let key = h.take().opens()[0].key;
    h.listed(&[&p]);
    let asked = h.ui_out(UiEvent::Choose {
        key,
        fingerprint: p.fingerprint,
        remember: true,
    });
    h.chains(&asked, &[]);
    let expected = websign_core::present::caller::consent_key(&common::harness::desktop_caller());
    assert_eq!(h.state.borrow().consent[0].key, expected);
}
