//! SPEC §8.11, §6 and D4: one request on screen, ten waiting, no batching,
//! and callers that never mix.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN, OTHER_ORIGIN, wire};
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::RequestKey;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

fn ids(n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("r{i}")).collect()
}

#[test]
fn one_active_and_ten_waiting_are_accepted_and_the_twelfth_is_busy() {
    let mut h = Harness::native_ready();
    let mut opens = 0;
    for id in ids(11) {
        let out = h.begin(&id, ORIGIN, SHA256);
        assert!(out.errors().is_empty(), "{id} was accepted");
        opens += out.opens().len();
    }
    assert_eq!(opens, 1, "only the first request reaches the screen");

    let out = h.begin("r12", ORIGIN, SHA256);
    assert_eq!(out.only_error(), ("r12".to_owned(), ErrorCode::Busy));
    assert!(
        out.opens().is_empty(),
        "a refused request never opens a window"
    );
    assert!(out.keys.is_empty());
}

#[test]
fn a_waiting_request_does_not_touch_the_key_store() {
    let mut h = Harness::native_ready();
    h.begin("a", ORIGIN, SHA256);
    let out = h.begin("b", ORIGIN, SHA256);
    assert!(out.lists().is_empty() && out.opens().is_empty());
}

#[test]
fn finishing_the_first_opens_the_second_with_its_place_in_line() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let mut first = None;
    for id in ids(11) {
        let out = h.begin(&id, ORIGIN, SHA256);
        first = first.or_else(|| out.opens().first().map(|o| o.key));
    }
    h.listed(&[&p]);

    let out = h.ui_out(UiEvent::Cancel {
        key: first.expect("first"),
        code: ErrorCode::UserCancelled,
    });
    assert_eq!(
        out.only_error(),
        ("r1".to_owned(), ErrorCode::UserCancelled)
    );
    let opens = out.opens();
    assert_eq!(opens.len(), 1, "the next request takes the window");
    assert_eq!(opens[0].position, (1, 10), "1 active + 9 waiting");
    assert_ne!(opens[0].key, first.expect("first"));
    assert_eq!(out.lists().len(), 1, "it lists for itself");
}

#[test]
fn requests_are_served_first_in_first_out() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let mut current = None;
    for id in ["a", "b", "c"] {
        let out = h.begin(id, ORIGIN, SHA256);
        current = current.or_else(|| out.opens().first().map(|o| o.key));
    }
    let mut current = current.expect("a is on screen");
    for expected in ["a", "b", "c"] {
        h.listed(&[&p]);
        let out = h.ui_out(UiEvent::Continue {
            key: current,
            fingerprint: p.fingerprint,
        });
        assert_eq!(
            out.need_digests()[0].0,
            expected,
            "the window serves {expected}"
        );
        let out = h.ui_out(UiEvent::Cancel {
            key: current,
            code: ErrorCode::UserCancelled,
        });
        if let Some(next) = out.opens().first() {
            current = next.key;
        }
    }
}

#[test]
fn a_waiting_request_can_be_cancelled_without_disturbing_the_active_one() {
    let mut h = Harness::native_ready();
    let first = h.begin("a", ORIGIN, SHA256).opens()[0].key;
    h.begin("b", ORIGIN, SHA256);
    h.send(wire::cancel("b"));
    let out = h.take();
    assert_eq!(out.only_error(), ("b".to_owned(), ErrorCode::Aborted));
    assert!(
        out.finished().iter().all(|(key, _)| *key != first),
        "a is still on screen"
    );
}

#[test]
fn the_window_is_told_when_the_line_changes() {
    let mut h = Harness::native_ready();
    let first = h.begin("a", ORIGIN, SHA256).opens()[0].key;
    let out = h.begin("b", ORIGIN, SHA256);
    // SPEC §6: the window is told whenever the line grows or shrinks.
    assert_eq!(
        out.ui,
        [websign_ui_model::confirm::UiCommand::Queue {
            key: first,
            position: (1, 2),
        }],
        "the eyebrow must say 1 of 2"
    );
}

#[test]
fn no_batching_each_request_needs_its_own_confirmation() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let a = h.begin("a", ORIGIN, SHA256).opens()[0].key;
    h.begin("b", ORIGIN, SHA256);
    h.listed(&[&p]);
    h.answer_digest("a", 1, SHA256);

    // Signing a does not sign b, and b has not even asked for a digest yet.
    let out = h.press_sign(a, &p, None);
    assert_eq!(out.signs().len(), 1);
    let out = h.signed_ok(out.signs()[0].tag, &p, SHA256, SignatureAlgorithm::Ecdsa);
    assert_eq!(out.results().len(), 1);
    assert_eq!(out.results()[0].0, "a");
    assert!(out.signs().is_empty());
    let opens = out.opens();
    assert_eq!(opens.len(), 1, "b takes the window");
    let b = opens[0].key;
    assert!(
        out.need_digests().is_empty(),
        "b's digest is asked only after b is listed"
    );

    h.listed(&[&p]);
    h.answer_digest("b", 1, SHA256);
    // A Ui Sign for the finished request is ignored, for b it is required.
    assert!(h.press_sign(a, &p, None).signs().is_empty());
    assert_eq!(h.press_sign(b, &p, None).signs().len(), 1);
}

#[test]
fn window_events_for_a_request_that_is_only_waiting_are_ignored() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.begin("a", ORIGIN, SHA256);
    h.begin("b", ORIGIN, SHA256);
    let waiting = RequestKey(u64::MAX - 1);
    h.listed(&[&p]);
    let out = h.ui_out(UiEvent::Continue {
        key: waiting,
        fingerprint: p.fingerprint,
    });
    assert!(out.need_digests().is_empty());
}

#[test]
fn callers_never_mix_a_remembered_site_does_not_lend_consent_to_the_next_one() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    let a = h.begin("a", ORIGIN, SHA256).opens()[0].key;
    assert!(h.begin("b", OTHER_ORIGIN, SHA256).opens().is_empty());

    h.listed(&[&p]);
    h.answer_digest("a", 1, SHA256);
    let tag = h.press_sign(a, &p, None).signs()[0].tag;
    let out = h.signed_ok(tag, &p, SHA256, SignatureAlgorithm::Ecdsa);
    let open_b = &out.opens()[0];
    assert!(!open_b.remembered, "the other origin is a new caller");

    let out = h.listed(&[&p]);
    assert!(out.need_digests().is_empty(), "D11 for the second caller");
    let out = h.ui_out(UiEvent::Continue {
        key: open_b.key,
        fingerprint: p.fingerprint,
    });
    assert_eq!(out.need_digests()[0].0, "b");
}

#[test]
fn a_status_and_a_remembered_choose_bypass_the_queue() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.remember(OTHER_ORIGIN, &[&p]);
    h.begin("a", ORIGIN, SHA256);

    h.send(wire::status("s", Some(ORIGIN)));
    assert_eq!(h.take().kinds(), ["status"]);

    let out = {
        h.send(wire::choose("c", Some(OTHER_ORIGIN)));
        h.take()
    };
    assert!(out.opens().is_empty(), "no window for a remembered choose");
    // One listing serves every request that waits for one: the choose is
    // answered from it while the sign request stays on screen, untouched.
    let out = h.listed_with_chains(&[&p]);
    assert_eq!(out.kinds(), ["choose.result"]);
    assert!(out.finished().is_empty());
}

#[test]
fn a_full_line_moves_up_when_a_waiting_request_is_cancelled() {
    let mut h = Harness::native_ready();
    for id in ids(11) {
        h.begin(&id, ORIGIN, SHA256);
    }
    h.take();
    h.send(wire::cancel("r5"));
    let out = h.take();
    assert_eq!(out.error_codes(), [ErrorCode::Aborted]);
    assert!(out.opens().is_empty(), "the active request is untouched");
    let out = h.begin("again", ORIGIN, SHA256);
    assert!(out.errors().is_empty(), "the freed place can be used");
}

#[test]
fn a_duplicate_id_is_refused_and_the_original_request_goes_on() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.begin("a", ORIGIN, SHA256);
    let out = h.begin("a", ORIGIN, SHA256);
    assert_eq!(
        out.only_error(),
        ("a".to_owned(), ErrorCode::InvalidRequest)
    );
    assert!(out.opens().is_empty());
    let out = h.listed(&[&p]);
    assert!(out.errors().is_empty(), "the first request is still open");
}
