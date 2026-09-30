//! SPEC §4 table, rows `Keys(Signed …)`, `Ui(Cancel)` and `end`, and §4.1.

mod common;

use common::Cert;
use common::effects::{failure, finish, sent_error, sign_command, words};
use common::flows::{KEY, awaiting, digest_message, press_sign, ready};
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_host::flow::Effect;
use websign_host::flow::sign::SignState;
use websign_host::ports::KeyReply;
use websign_keystores::KeystoreError;
use websign_protocol::{AppMessage, ErrorCode};
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::{Failure, Finish};

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

fn signed(tag: u64, result: Result<Vec<u8>, KeystoreError>) -> KeyReply {
    KeyReply::Signed { tag, result }
}

/// A flow that pressed Sign, with the tag of its key store call.
fn signing(cert: &Cert) -> (websign_host::flow::sign::SignFlow, u64) {
    let mut flow = ready(cert);
    let effects = press_sign(&mut flow, cert);
    let (tag, _, _) = sign_command(&effects).expect("a Sign command");
    (flow, tag)
}

#[test]
fn a_verified_signature_finishes_with_the_result() {
    let p = Cert::p256();
    let (mut flow, tag) = signing(&p);
    let signature = p.signature(SHA256, SignatureAlgorithm::Ecdsa);
    let effects = flow.on_keys(&signed(tag, Ok(signature.clone())));
    assert_eq!(flow.state, SignState::Done);
    assert!(words(&effects).contains(&"consent".to_owned()));
    assert_eq!(finish(&effects), Some(Finish::Signed));
    let sent: Vec<_> = effects
        .iter()
        .filter_map(|e| match e {
            Effect::Send(AppMessage::SignResult(r)) => Some(r.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].signature.as_bytes(), signature.as_slice());
    let position = |w: &str| words(&effects).iter().position(|x| x == w).expect(w);
    assert!(position("send:sign_result") < position("ui:finished"));
}

#[test]
fn the_consent_effect_carries_the_fingerprint_and_the_remember_choice() {
    let p = Cert::p256();
    let mut flow = ready(&p);
    let effects = flow.on_ui(UiEvent::Sign {
        key: KEY,
        fingerprint: p.fingerprint,
        via: 0,
        pin: None,
        remember: true,
    });
    let (tag, _, _) = sign_command(&effects).expect("sign");
    let effects = flow.on_keys(&signed(
        tag,
        Ok(p.signature(SHA256, SignatureAlgorithm::Ecdsa)),
    ));
    let recorded = effects.iter().find_map(|e| match e {
        Effect::RecordConsent {
            remember,
            fingerprint,
        } => Some((*remember, fingerprint.clone())),
        _ => None,
    });
    assert_eq!(recorded, Some((true, p.hex())));
}

#[test]
fn a_signature_that_does_not_verify_is_never_sent() {
    let p = Cert::p256();
    let (mut flow, tag) = signing(&p);
    let effects = flow.on_keys(&signed(tag, Ok(vec![5; 64])));
    assert!(sent_error(&effects).is_none());
    assert!(!words(&effects).contains(&"send:sign_result".to_owned()));
    match failure(&effects) {
        Some(Failure::DriverFailure { native, .. }) => {
            assert_eq!(native, "signature did not verify")
        }
        other => panic!("expected DriverFailure, got {other:?}"),
    }
    assert!(words(&effects).contains(&"error".to_owned()));
    assert!(matches!(flow.state, SignState::Ready { .. }));
}

#[test]
fn key_store_errors_map_to_states_and_window_failures() {
    type Check = fn(&Option<Failure>) -> bool;
    let cases: Vec<(&str, KeystoreError, Check, bool, bool)> = vec![
        (
            "wrong pin",
            KeystoreError::WrongPin,
            |f| matches!(f, Some(Failure::PinIncorrect { .. })),
            true,
            false,
        ),
        (
            "pin locked",
            KeystoreError::PinLocked,
            |f| matches!(f, Some(Failure::PinLocked { .. })),
            false,
            true,
        ),
        (
            "token removed",
            KeystoreError::TokenRemoved,
            |f| matches!(f, Some(Failure::TokenRemoved)),
            false,
            false,
        ),
        (
            "not found",
            KeystoreError::NotFound,
            |f| matches!(f, Some(Failure::CertificateUnavailable)),
            false,
            false,
        ),
        (
            "unsupported",
            KeystoreError::Unsupported("PSS".to_owned()),
            |f| matches!(f, Some(Failure::UnsupportedAlgorithm { .. })),
            false,
            false,
        ),
        (
            "other",
            KeystoreError::Other("x".to_owned()),
            |f| matches!(f, Some(Failure::DriverFailure { .. })),
            true,
            true,
        ),
    ];
    for (name, error, check, back_to_ready, records_error) in cases {
        let p = Cert::p256();
        let (mut flow, tag) = signing(&p);
        let effects = flow.on_keys(&signed(tag, Err(error)));
        assert!(check(&failure(&effects)), "{name}: {:?}", words(&effects));
        assert!(
            sent_error(&effects).is_none(),
            "{name}: the caller hears nothing yet"
        );
        assert_eq!(
            matches!(flow.state, SignState::Ready { .. }),
            back_to_ready,
            "{name}: {:?}",
            flow.state
        );
        if !back_to_ready {
            assert!(
                matches!(flow.state, SignState::Selecting { .. }),
                "{name}: {:?}",
                flow.state
            );
        }
        assert_eq!(
            words(&effects).contains(&"error".to_owned()),
            records_error,
            "{name}"
        );
    }
}

#[test]
fn a_native_error_is_a_driver_failure_that_is_recorded() {
    let p = Cert::p256();
    let (mut flow, tag) = signing(&p);
    let error = KeystoreError::Native {
        api: "C_Sign",
        code: 0x30,
        message: "device".to_owned(),
    };
    let effects = flow.on_keys(&signed(tag, Err(error)));
    assert!(matches!(
        failure(&effects),
        Some(Failure::DriverFailure { .. })
    ));
    assert!(words(&effects).contains(&"error".to_owned()));
    assert!(matches!(flow.state, SignState::Ready { .. }));
}

#[test]
fn cancelling_the_os_dialog_returns_to_ready_without_a_message() {
    let p = Cert::p256();
    let (mut flow, tag) = signing(&p);
    let effects = flow.on_keys(&signed(tag, Err(KeystoreError::Cancelled)));
    assert!(effects.is_empty(), "{:?}", words(&effects));
    assert!(matches!(flow.state, SignState::Ready { .. }));
}

#[test]
fn a_reply_with_another_tag_is_ignored() {
    let p = Cert::p256();
    let (mut flow, tag) = signing(&p);
    let before = flow.state.clone();
    let effects = flow.on_keys(&signed(tag + 1, Err(KeystoreError::WrongPin)));
    assert!(effects.is_empty());
    assert_eq!(flow.state, before);
}

#[test]
fn ui_cancel_ends_with_the_windows_code() {
    let p = Cert::p256();
    for code in [
        ErrorCode::UserCancelled,
        ErrorCode::NoCertificates,
        ErrorCode::PinLocked,
    ] {
        let mut flow = awaiting(&p);
        let effects = flow.on_ui(UiEvent::Cancel { key: KEY, code });
        assert_eq!(sent_error(&effects), Some(code));
        assert!(finish(&effects).is_some());
        assert_eq!(flow.state, SignState::Done);
    }
}

#[test]
fn end_reports_the_code_and_tells_the_window_why() {
    let p = Cert::p256();
    for (code, expected) in [
        (ErrorCode::Aborted, Finish::Aborted),
        (ErrorCode::Timeout, Finish::Timeout),
    ] {
        let mut flow = awaiting(&p);
        let effects = flow.end(code);
        assert_eq!(sent_error(&effects), Some(code));
        assert_eq!(finish(&effects), Some(expected));
        assert_eq!(flow.state, SignState::Done);
    }
}

#[test]
fn a_finished_flow_does_nothing_more() {
    let p = Cert::p256();
    let mut flow = awaiting(&p);
    flow.end(ErrorCode::Aborted);
    assert!(flow.end(ErrorCode::Timeout).is_empty(), "no second error");
    assert!(flow.on_digest(digest_message(1, vec![0; 32])).is_empty());
    assert!(press_sign(&mut flow, &p).is_empty());
    assert!(
        flow.on_ui(UiEvent::Cancel {
            key: KEY,
            code: ErrorCode::UserCancelled
        })
        .is_empty()
    );
}
