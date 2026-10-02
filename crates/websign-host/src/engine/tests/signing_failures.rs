//! Scenarios 7 and 8, and the other outcomes a key store can report.

use websign_keystores::KeystoreError;
use websign_protocol::AppMessage;
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::Failure;

use super::rig::Rig;
use crate::ports::KeyReply;
use crate::testing::fixture;

/// A request with its digest on screen and Sign pressed; returns the tag.
fn signing(rig: &mut Rig) -> (websign_ui_model::confirm::port::RequestKey, u64) {
    let key = rig.until_ready("1");
    rig.h.keys.take();
    rig.h.ui.take();
    rig.h.outbound.take();
    rig.press_sign(key, false);
    (key, rig.sign_tag())
}

fn failures(rig: &Rig) -> Vec<Failure> {
    rig.h
        .ui
        .take()
        .into_iter()
        .filter_map(|command| match command {
            UiCommand::Failed { failure, .. } => Some(failure),
            _ => None,
        })
        .collect()
}

#[test]
fn a_wrong_pin_stays_in_the_window_and_the_next_try_signs() {
    let mut rig = Rig::browser();
    let (key, tag) = signing(&mut rig);
    rig.keys(KeyReply::Signed {
        tag,
        result: Err(KeystoreError::WrongPin),
    });
    assert!(matches!(
        failures(&rig).as_slice(),
        [Failure::PinIncorrect { .. }]
    ));
    assert!(rig.messages().is_empty(), "the caller hears nothing");

    rig.press_sign(key, false);
    let tag = rig.sign_tag();
    rig.keys(KeyReply::Signed {
        tag,
        result: Ok(fixture::signature()),
    });
    let sent = rig.messages();
    assert!(matches!(&sent[..], [AppMessage::SignResult(_)]));
}

#[test]
fn a_signature_that_does_not_verify_is_never_sent() {
    let mut rig = Rig::browser();
    let (_, tag) = signing(&mut rig);
    rig.keys(KeyReply::Signed {
        tag,
        result: Ok(vec![0; 64]),
    });
    assert!(rig.messages().is_empty());
    let failed = failures(&rig);
    assert!(matches!(
        failed.as_slice(),
        [Failure::DriverFailure { native, .. }] if native == "signature did not verify"
    ));
    let errors = rig.engine.ports.stores.errors();
    let recorded = errors.list().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].code, "DriverFailure");
}

#[test]
fn a_locked_pin_returns_to_the_list_and_is_recorded() {
    let mut rig = Rig::browser();
    let (_, tag) = signing(&mut rig);
    rig.keys(KeyReply::Signed {
        tag,
        result: Err(KeystoreError::PinLocked),
    });
    assert!(matches!(
        failures(&rig).as_slice(),
        [Failure::PinLocked { .. }]
    ));
    assert_eq!(rig.engine.ports.stores.errors().list().unwrap().len(), 1);
}

#[test]
fn removal_and_unsupported_algorithms_return_to_the_list() {
    let mut rig = Rig::browser();
    let (_, tag) = signing(&mut rig);
    rig.keys(KeyReply::Signed {
        tag,
        result: Err(KeystoreError::TokenRemoved),
    });
    assert!(matches!(failures(&rig).as_slice(), [Failure::TokenRemoved]));

    let mut rig = Rig::browser();
    let (_, tag) = signing(&mut rig);
    rig.keys(KeyReply::Signed {
        tag,
        result: Err(KeystoreError::Unsupported("pss".into())),
    });
    assert!(matches!(
        failures(&rig).as_slice(),
        [Failure::UnsupportedAlgorithm { .. }]
    ));
}

#[test]
fn an_os_dialog_cancelled_goes_back_silently() {
    let mut rig = Rig::browser();
    let (key, tag) = signing(&mut rig);
    rig.h.ui.take();
    rig.keys(KeyReply::Signed {
        tag,
        result: Err(KeystoreError::Cancelled),
    });
    assert!(rig.h.ui.take().is_empty());
    rig.press_sign(key, false);
    assert!(!rig.h.keys.is_empty(), "the person can sign again");
}

#[test]
fn native_errors_are_driver_failures_with_the_status_recorded() {
    let mut rig = Rig::browser();
    let (_, tag) = signing(&mut rig);
    rig.keys(KeyReply::Signed {
        tag,
        result: Err(KeystoreError::Native {
            api: "C_Sign",
            code: 0x30,
            message: "CKR_DEVICE_ERROR".into(),
        }),
    });
    assert!(matches!(
        failures(&rig).as_slice(),
        [Failure::DriverFailure { native, .. }] if native.starts_with("CKR_DEVICE_ERROR")
    ));
    let recorded = rig.engine.ports.stores.errors().list().unwrap();
    assert_eq!(recorded[0].source, "pkcs11");
}

#[test]
fn a_result_after_the_request_ended_is_dropped() {
    let mut rig = Rig::browser();
    let (_, tag) = signing(&mut rig);
    rig.frame(r#""id":"1","type":"cancel""#);
    rig.h.outbound.take();
    rig.keys(KeyReply::Signed {
        tag,
        result: Ok(fixture::signature()),
    });
    assert!(rig.messages().is_empty());
}

#[test]
fn the_window_handle_reaches_the_key_store() {
    let mut rig = Rig::browser();
    rig.h.ui.set_parent_window(Some(42));
    let key = rig.until_ready("1");
    rig.h.keys.take();
    rig.press_sign(key, false);
    let sign = rig
        .h
        .keys
        .take()
        .into_iter()
        .find_map(|command| match command {
            crate::ports::KeyCommand::Sign { parent_window, .. } => Some(parent_window),
            _ => None,
        });
    assert_eq!(sign, Some(Some(42)));
}
