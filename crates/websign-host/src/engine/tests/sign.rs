//! Scenarios 3 to 6, 9 (cancel) and 10: the happy path, consent, sequence
//! numbers, digest length and deadlines.

use std::collections::HashMap;
use std::time::Duration;

use websign_core::Fingerprint;
use websign_protocol::{AppMessage, ErrorCode};
use websign_ui_model::confirm::port::{Finish, Mode};
use websign_ui_model::confirm::{UiCommand, UiEvent};

use super::rig::{Rig, fp};
use crate::engine::Control;
use crate::ports::{KeyCommand, KeyReply};
use crate::testing::fixture;

fn error_codes(messages: &[AppMessage]) -> Vec<ErrorCode> {
    messages
        .iter()
        .filter_map(|message| match message {
            AppMessage::Error(error) => Some(error.code),
            _ => None,
        })
        .collect()
}

/// Signs once with "Remember" ticked, so the site is remembered afterwards.
fn remember_the_site(rig: &mut Rig) {
    let key = rig.until_ready("first");
    rig.h.keys.take();
    rig.press_sign(key, true);
    let tag = rig.sign_tag();
    rig.keys(KeyReply::Signed {
        tag,
        result: Ok(fixture::signature()),
    });
    assert!(matches!(
        rig.messages().last(),
        Some(AppMessage::SignResult(_))
    ));
    rig.h.ui.take();
}

#[test]
fn a_new_site_signs_after_continue_and_the_result_carries_the_verified_signature() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let commands = rig.h.ui.take();
    let open = Rig::opened(&commands).expect("the window opens");
    assert_eq!(
        open.mode,
        Mode::Sign {
            hash: websign_protocol::types::HashName::Sha256
        }
    );
    assert!(!open.remembered);
    assert_eq!(open.position, (1, 1));
    assert!(matches!(
        rig.h.keys.take().as_slice(),
        [KeyCommand::List { refresh: false }]
    ));

    rig.listed(fixture::snapshot());
    assert!(
        rig.messages().is_empty(),
        "nothing is released before Continue"
    );
    rig.ui(UiEvent::Continue {
        key: open.key,
        fingerprint: fp(),
    });
    assert!(rig.messages().is_empty(), "need_digest waits for the chain");
    rig.answer_chains();
    let need = rig.messages();
    assert!(
        matches!(&need[..], [AppMessage::NeedDigest(need)] if need.seq == 1 && need.certificate.fingerprint.as_str() == fp().to_hex())
    );
    rig.digest("1", 1, &fixture::DIGEST);
    assert!(
        rig.h
            .ui
            .take()
            .iter()
            .any(|c| matches!(c, UiCommand::DigestReady { .. }))
    );

    rig.h.keys.take();
    rig.press_sign(open.key, false);
    let tag = rig.sign_tag();
    rig.keys(KeyReply::Signed {
        tag,
        result: Ok(fixture::signature()),
    });
    let sent = rig.messages();
    assert!(
        matches!(&sent[..], [AppMessage::SignResult(result)] if result.hash == websign_protocol::types::HashName::Sha256)
    );
    let ui = rig.h.ui.take();
    assert!(ui.contains(&UiCommand::Finished {
        key: open.key,
        finish: Finish::Signed
    }));
}

#[test]
fn a_remembered_site_gets_need_digest_without_continue_and_the_window_knows() {
    let mut rig = Rig::browser();
    remember_the_site(&mut rig);

    rig.sign_begin("2");
    let open = Rig::opened(&rig.h.ui.take()).unwrap();
    assert!(open.remembered);
    assert_eq!(open.consented, [fp()]);
    rig.listed(fixture::snapshot());
    rig.answer_chains();
    let messages = rig.messages();
    assert!(matches!(&messages[..], [AppMessage::NeedDigest(need)] if need.seq == 1));
    assert!(
        rig.h
            .ui
            .take()
            .iter()
            .any(|c| matches!(c, UiCommand::DigestPending { .. }))
    );
}

#[test]
fn cancelling_before_continue_leaves_the_caller_with_no_certificate() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(fixture::snapshot());
    rig.ui(UiEvent::Cancel {
        key,
        code: ErrorCode::UserCancelled,
    });
    let sent = rig.messages();
    assert_eq!(error_codes(&sent), [ErrorCode::UserCancelled]);
    assert_eq!(sent.len(), 1, "no need_digest, so no certificate");
}

#[test]
fn switching_to_a_certificate_outside_the_consent_waits_for_continue() {
    let mut rig = Rig::browser();
    remember_the_site(&mut rig);
    let second = Fingerprint::from_bytes([7; 32]);
    let mut snapshot = fixture::snapshot();
    let mut other = fixture::candidate();
    other.fingerprint = second;
    snapshot.candidates.push(other);
    snapshot.certificates = HashMap::from([(fp(), fixture::der()), (second, fixture::der())]);

    rig.sign_begin("2");
    let key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(snapshot);
    rig.answer_chains();
    assert!(matches!(&rig.messages()[..], [AppMessage::NeedDigest(n)] if n.seq == 1));

    rig.ui(UiEvent::Selected {
        key,
        fingerprint: second,
    });
    rig.answer_chains();
    assert!(
        rig.messages().is_empty(),
        "moving the selection discloses nothing"
    );
    rig.ui(UiEvent::Continue {
        key,
        fingerprint: second,
    });
    rig.answer_chains();
    assert!(matches!(&rig.messages()[..], [AppMessage::NeedDigest(n)] if n.seq == 2));
    rig.h.ui.take();

    rig.digest("2", 1, &fixture::DIGEST);
    assert!(rig.h.ui.take().is_empty(), "the late digest is ignored");
    rig.digest("2", 2, &fixture::DIGEST);
    assert!(rig.h.ui.take().iter().any(|c| matches!(
        c,
        UiCommand::DigestReady { fingerprint, .. } if *fingerprint == second
    )));
}

#[test]
fn a_wrong_digest_length_ends_with_invalid_request() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(fixture::snapshot());
    rig.ui(UiEvent::Continue {
        key,
        fingerprint: fp(),
    });
    rig.h.outbound.take();
    rig.digest("1", 1, &[1; 20]);
    assert_eq!(error_codes(&rig.messages()), [ErrorCode::InvalidRequest]);
    assert!(
        rig.h
            .ui
            .take()
            .iter()
            .any(|c| matches!(c, UiCommand::Failed { .. }))
    );
}

#[test]
fn a_cancel_from_the_client_ends_with_aborted() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    rig.frame(r#""id":"1","type":"cancel""#);
    assert_eq!(error_codes(&rig.messages()), [ErrorCode::Aborted]);
    // A cancel that crosses the final reply is not an error.
    rig.frame(r#""id":"1","type":"cancel""#);
    assert!(rig.messages().is_empty());
}

#[test]
fn the_decision_deadline_ends_the_request_with_timeout() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    assert_eq!(rig.tick_after(Duration::from_secs(299)), Control::Continue);
    assert!(rig.messages().is_empty());
    rig.tick_after(Duration::from_secs(2));
    assert_eq!(error_codes(&rig.messages()), [ErrorCode::Timeout]);
    assert!(rig.h.ui.take().contains(&UiCommand::Finished {
        key,
        finish: Finish::Timeout
    }));
}

#[test]
fn the_digest_deadline_runs_from_each_need_digest() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(fixture::snapshot());
    rig.tick_after(Duration::from_secs(50));
    rig.ui(UiEvent::Continue {
        key,
        fingerprint: fp(),
    });
    rig.answer_chains();
    rig.h.outbound.take();
    rig.tick_after(Duration::from_secs(59));
    assert!(rig.messages().is_empty());
    rig.tick_after(Duration::from_secs(2));
    assert_eq!(error_codes(&rig.messages()), [ErrorCode::Timeout]);
}

#[test]
fn events_for_a_request_that_is_not_on_screen_are_ignored() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(fixture::snapshot());
    rig.ui(UiEvent::Continue {
        key: websign_ui_model::confirm::port::RequestKey(999),
        fingerprint: fp(),
    });
    assert!(rig.messages().is_empty());
    rig.ui(UiEvent::Rescan { key });
    let commands = rig.h.keys.take();
    assert!(matches!(
        &commands[..],
        [
            KeyCommand::List { refresh: false },
            KeyCommand::Invalidate,
            KeyCommand::List { refresh: true }
        ]
    ));
}
