//! SPEC §4 table, rows `activate` to `Ui(Continue)`: the sign flow's
//! effects, driven without the engine.

mod common;

use std::time::{Duration, Instant};

use common::Cert;
use common::effects::{need_digest_seq, words};
use common::flows::{KEY, listed, new_flow};
use websign_host::flow::Effect;
use websign_host::flow::sign::SignState;
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::Mode;

#[test]
fn a_new_flow_is_queued_without_a_deadline() {
    let flow = new_flow(false);
    assert_eq!(flow.state, SignState::Queued);
    assert_eq!(flow.deadline, None);
}

#[test]
fn activate_opens_the_window_lists_and_starts_the_300_second_clock() {
    let mut flow = new_flow(true);
    let now = Instant::now();
    let effects = flow.activate(now, &common::flows::presentation());
    assert_eq!(words(&effects), ["ui:open", "keys:list"]);
    assert_eq!(flow.state, SignState::Listing);
    assert_eq!(flow.deadline, Some(now + Duration::from_secs(300)));
    match &effects[0] {
        Effect::Ui(UiCommand::Open(open)) => {
            assert_eq!(open.key, KEY);
            assert!(open.remembered);
            assert!(matches!(open.mode, Mode::Sign { .. }));
            assert_eq!(open.timeout_secs, 300);
        }
        other => panic!("expected Open, got {other:?}"),
    }
    assert!(matches!(
        &effects[1],
        Effect::Keys(websign_host::ports::KeyCommand::List { refresh: false })
    ));
}

#[test]
fn a_new_caller_sees_the_list_and_nothing_is_released() {
    let p = Cert::p256();
    let flow = listed(false, &[&p]);
    assert!(
        matches!(flow.state, SignState::Selecting { .. }),
        "{:?}",
        flow.state
    );

    let mut flow = new_flow(false);
    flow.activate(Instant::now(), &common::flows::presentation());
    let effects = flow.on_listed(&common::snapshot(&[&p]), common::flows::context());
    assert_eq!(words(&effects), ["ui:certificates"]);
}

#[test]
fn a_remembered_caller_with_a_preselected_certificate_is_released_at_once() {
    let p = Cert::p256();
    let mut flow = new_flow(true);
    flow.activate(Instant::now(), &common::flows::presentation());
    let effects = flow.on_listed(&common::snapshot(&[&p]), common::flows::context());
    assert_eq!(
        words(&effects),
        [
            "ui:certificates",
            "send:need_digest",
            "ui:digest_pending",
            "keys:chain"
        ]
    );
    assert_eq!(need_digest_seq(&effects), Some(1));
    assert_eq!(
        flow.state,
        SignState::AwaitingDigest {
            seq: 1,
            fingerprint: p.fingerprint
        }
    );
}

#[test]
fn nothing_usable_keeps_selecting_and_the_window_shows_empty() {
    for remembered in [false, true] {
        let mut flow = new_flow(remembered);
        flow.activate(Instant::now(), &common::flows::presentation());
        let effects = flow.on_listed(&common::snapshot(&[]), common::flows::context());
        assert_eq!(
            words(&effects),
            ["ui:certificates"],
            "remembered={remembered}"
        );
        assert!(matches!(flow.state, SignState::Selecting { .. }));
    }
}

#[test]
fn a_second_listing_only_refreshes_the_window() {
    let p = Cert::p256();
    let mut flow = listed(true, &[&p]);
    let before = flow.state.clone();
    let effects = flow.on_listed(&common::snapshot(&[&p]), common::flows::context());
    assert_eq!(words(&effects), ["ui:certificates"]);
    assert_eq!(flow.state, before);

    let mut flow = listed(false, &[&p]);
    let effects = flow.on_listed(&common::snapshot(&[&p]), common::flows::context());
    assert_eq!(words(&effects), ["ui:certificates"]);
}

#[test]
fn continue_releases_the_certificate_of_a_new_caller() {
    let p = Cert::p256();
    let mut flow = listed(false, &[&p]);
    let effects = flow.on_ui(UiEvent::Continue {
        key: KEY,
        fingerprint: p.fingerprint,
    });
    assert_eq!(
        words(&effects),
        ["send:need_digest", "ui:digest_pending", "keys:chain"],
        "the chain is read for the certificate just released"
    );
    assert_eq!(need_digest_seq(&effects), Some(1));
    assert_eq!(
        flow.state,
        SignState::AwaitingDigest {
            seq: 1,
            fingerprint: p.fingerprint
        }
    );
}

#[test]
fn selecting_for_a_new_caller_only_moves_the_selection() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut flow = listed(false, &[&p, &r]);
    let effects = flow.on_ui(UiEvent::Selected {
        key: KEY,
        fingerprint: r.fingerprint,
    });
    assert!(effects.is_empty(), "{:?}", words(&effects));
    assert_eq!(
        flow.state,
        SignState::Selecting {
            selected: Some(r.fingerprint)
        }
    );
}

#[test]
fn selecting_for_a_remembered_caller_asks_for_a_new_digest() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut flow = listed(true, &[&p, &r]);
    let effects = flow.on_ui(UiEvent::Selected {
        key: KEY,
        fingerprint: r.fingerprint,
    });
    assert_eq!(
        words(&effects),
        ["send:need_digest", "ui:digest_pending", "keys:chain"],
        "the chain is read for the certificate just released"
    );
    assert_eq!(need_digest_seq(&effects), Some(2));
    assert_eq!(
        flow.state,
        SignState::AwaitingDigest {
            seq: 2,
            fingerprint: r.fingerprint
        }
    );
}

#[test]
fn continue_for_a_certificate_that_was_not_listed_is_ignored() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut flow = listed(false, &[&p]);
    let before = flow.state.clone();
    let effects = flow.on_ui(UiEvent::Continue {
        key: KEY,
        fingerprint: r.fingerprint,
    });
    assert!(effects.is_empty());
    assert_eq!(flow.state, before);
}

#[test]
fn ui_events_are_ignored_before_the_request_is_on_screen() {
    let p = Cert::p256();
    let mut flow = new_flow(false);
    let effects = flow.on_ui(UiEvent::Continue {
        key: KEY,
        fingerprint: p.fingerprint,
    });
    assert!(effects.is_empty());
    assert_eq!(flow.state, SignState::Queued);
}

#[test]
fn the_context_decides_what_a_remembered_caller_gets_first() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    for (requested, last_used, expected) in [
        (None, Some(r.fingerprint), r.fingerprint),
        (Some(p.fingerprint), Some(r.fingerprint), p.fingerprint),
    ] {
        let mut context = common::flows::context();
        context.requested = requested;
        context.last_used_here = last_used;
        let mut flow = new_flow(true);
        flow.activate(Instant::now(), &common::flows::presentation());
        flow.on_listed(&common::snapshot(&[&p, &r]), context);
        assert_eq!(
            flow.state,
            SignState::AwaitingDigest {
                seq: 1,
                fingerprint: expected
            }
        );
    }
}
