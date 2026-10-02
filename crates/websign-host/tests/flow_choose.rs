//! SPEC §5: the choose flow's effects, driven without the engine.

mod common;

use std::time::Instant;

use common::Cert;
use common::effects::{finish, sent_error, words};
use websign_host::flow::Effect;
use websign_host::flow::choose::{ChooseFlow, ChooseState};
use websign_host::ports::{KeyCommand, KeyReply};
use websign_protocol::messages::Choose;
use websign_protocol::{AppMessage, ErrorCode};
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::{Finish, Mode, RequestKey};

const KEY: RequestKey = RequestKey(3);

fn flow(remembered: &[&Cert]) -> ChooseFlow {
    ChooseFlow::new(
        KEY,
        Choose::default(),
        remembered.iter().map(|c| c.hex()).collect(),
    )
}

fn sent_fingerprints(effects: &[Effect]) -> Vec<String> {
    effects
        .iter()
        .find_map(|e| match e {
            Effect::Send(AppMessage::ChooseResult(r)) => Some(
                r.certificates
                    .iter()
                    .map(|c| c.fingerprint.as_str().to_owned())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn only_remembered_callers_skip_the_window() {
    let p = Cert::p256();
    assert!(flow(&[&p]).answers_without_window());
    assert!(!flow(&[]).answers_without_window());
}

#[test]
fn a_new_flow_is_queued() {
    let f = flow(&[]);
    assert_eq!(f.state, ChooseState::Queued);
    assert_eq!(f.deadline, None);
}

/// The key store answers every chain lookup in `effects` with no chain.
fn answer_chains(f: &mut ChooseFlow, effects: &[Effect]) -> Vec<Effect> {
    let mut replies = Vec::new();
    for effect in effects {
        if let Effect::Keys(KeyCommand::Chain { tag, .. }) = effect {
            replies.extend(f.on_keys(&KeyReply::Chain {
                tag: *tag,
                chain: Vec::new(),
            }));
        }
    }
    replies
}

#[test]
fn a_remembered_choose_lists_then_answers_with_the_present_ones_most_recent_first() {
    let (p, r, b) = (Cert::p256(), Cert::rsa(), Cert::rsa_b());
    let mut f = flow(&[&b, &p, &r]);
    let effects = f.activate(Instant::now(), &common::flows::presentation());
    assert_eq!(words(&effects), ["keys:list"], "windowless: no Open");

    let effects = f.on_listed(
        &common::certs::snapshot(&[&p, &b]),
        common::flows::context(),
    );
    assert_eq!(words(&effects), ["keys:chain", "keys:chain"]);
    assert_eq!(f.state, ChooseState::Listing);
    let effects = answer_chains(&mut f, &effects);
    assert_eq!(words(&effects), ["send:choose_result"]);
    assert_eq!(sent_fingerprints(&effects), [b.hex(), p.hex()]);
    assert_eq!(f.state, ChooseState::Done);
}

#[test]
fn a_remembered_choose_with_nothing_present_goes_back_to_the_queue_as_a_new_caller() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut f = flow(&[&p]);
    f.activate(Instant::now(), &common::flows::presentation());
    let effects = f.on_listed(&common::certs::snapshot(&[&r]), common::flows::context());
    assert!(effects.is_empty(), "the engine queues it for the window");
    assert_eq!(f.state, ChooseState::Queued);
    assert_eq!(f.deadline, None);
    assert!(!f.answers_without_window());

    let effects = f.activate(Instant::now(), &common::flows::presentation());
    assert_eq!(words(&effects), ["ui:open", "keys:list"]);
}

#[test]
fn a_new_caller_opens_the_window_in_choose_mode_and_shows_the_list() {
    let p = Cert::p256();
    let mut f = flow(&[]);
    let effects = f.activate(Instant::now(), &common::flows::presentation());
    assert_eq!(words(&effects), ["ui:open", "keys:list"]);
    match &effects[0] {
        Effect::Ui(UiCommand::Open(open)) => {
            assert_eq!(open.key, KEY);
            assert!(matches!(open.mode, Mode::Choose));
            assert!(!open.remembered);
        }
        other => panic!("expected Open, got {other:?}"),
    }
    assert!(f.deadline.is_some());

    let effects = f.on_listed(&common::certs::snapshot(&[&p]), common::flows::context());
    assert_eq!(words(&effects), ["ui:certificates"]);
    assert_eq!(f.state, ChooseState::Choosing);
}

#[test]
fn choosing_reads_the_chain_then_records_consent_and_answers_with_exactly_that_certificate() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut f = flow(&[]);
    f.activate(Instant::now(), &common::flows::presentation());
    f.on_listed(
        &common::certs::snapshot(&[&p, &r]),
        common::flows::context(),
    );
    let effects = f.on_ui(UiEvent::Choose {
        key: KEY,
        fingerprint: r.fingerprint,
        remember: true,
    });
    assert_eq!(words(&effects), ["keys:chain"]);
    let effects = answer_chains(&mut f, &effects);
    assert_eq!(
        words(&effects),
        ["consent", "send:choose_result", "ui:finished"]
    );
    assert_eq!(sent_fingerprints(&effects), [r.hex()]);
    assert_eq!(finish(&effects), Some(Finish::Chosen));
    let consent = effects.iter().find_map(|e| match e {
        Effect::RecordConsent {
            remember,
            fingerprint,
        } => Some((*remember, fingerprint.clone())),
        _ => None,
    });
    assert_eq!(consent, Some((true, r.hex())));
    assert_eq!(f.state, ChooseState::Done);
}

#[test]
fn a_chain_for_another_tag_or_after_the_end_is_ignored() {
    let p = Cert::p256();
    let mut f = flow(&[&p]);
    f.activate(Instant::now(), &common::flows::presentation());
    f.on_listed(&common::certs::snapshot(&[&p]), common::flows::context());
    let stray = KeyReply::Chain {
        tag: u64::MAX,
        chain: Vec::new(),
    };
    assert!(f.on_keys(&stray).is_empty());
    f.end(ErrorCode::Aborted);
    assert!(f.on_keys(&stray).is_empty());
}

#[test]
fn choosing_an_unlisted_certificate_is_ignored() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut f = flow(&[]);
    f.activate(Instant::now(), &common::flows::presentation());
    f.on_listed(&common::certs::snapshot(&[&p]), common::flows::context());
    let effects = f.on_ui(UiEvent::Choose {
        key: KEY,
        fingerprint: r.fingerprint,
        remember: false,
    });
    assert!(effects.is_empty());
    assert_eq!(f.state, ChooseState::Choosing);
}

#[test]
fn cancel_and_end_report_a_code() {
    let p = Cert::p256();
    let mut f = flow(&[]);
    f.activate(Instant::now(), &common::flows::presentation());
    f.on_listed(&common::certs::snapshot(&[&p]), common::flows::context());
    let effects = f.on_ui(UiEvent::Cancel {
        key: KEY,
        code: ErrorCode::UserCancelled,
    });
    assert_eq!(sent_error(&effects), Some(ErrorCode::UserCancelled));
    assert_eq!(f.state, ChooseState::Done);

    let mut f = flow(&[]);
    f.activate(Instant::now(), &common::flows::presentation());
    let effects = f.end(ErrorCode::Timeout);
    assert_eq!(sent_error(&effects), Some(ErrorCode::Timeout));
    assert_eq!(finish(&effects), Some(Finish::Timeout));
}

#[test]
fn a_finished_choose_does_nothing_more() {
    let mut f = flow(&[]);
    f.activate(Instant::now(), &common::flows::presentation());
    f.end(ErrorCode::Aborted);
    assert!(f.end(ErrorCode::Timeout).is_empty());
}
