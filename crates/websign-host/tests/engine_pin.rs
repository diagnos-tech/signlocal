//! SPEC §8.7, §4 (key store replies) and T6: PIN errors stay in the window and
//! the PIN never travels toward the page.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN};
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_keystores::KeystoreError;
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::{Failure, RequestKey};

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;
const ECDSA: SignatureAlgorithm = SignatureAlgorithm::Ecdsa;
const PIN: &str = "482913";

fn ready() -> (Harness, Cert, RequestKey, u64) {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let tag = h.press_sign(key, &p, Some(PIN)).signs()[0].tag;
    (h, p, key, tag)
}

#[test]
fn wrong_pin_then_right_pin_gives_one_result_and_no_error_frame() {
    let (mut h, p, key, tag) = ready();

    let out = h.signed_err(tag, KeystoreError::WrongPin);
    assert!(
        out.frames.is_empty(),
        "the page must not learn about a wrong PIN"
    );
    assert!(matches!(
        out.failures().as_slice(),
        [Failure::PinIncorrect { .. }]
    ));

    let out = h.press_sign(key, &p, Some("135790"));
    let second = out.signs();
    assert_eq!(second.len(), 1, "back in Ready, Sign works again");
    assert_ne!(second[0].tag, tag, "a new attempt is a new call");
    assert_eq!(second[0].pin.as_deref(), Some("135790"));

    let out = h.signed_ok(second[0].tag, &p, SHA256, ECDSA);
    assert_eq!(out.results().len(), 1);
    assert!(out.errors().is_empty());
}

#[test]
fn the_pin_reaches_the_key_store_and_nothing_else() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);

    let out = h.press_sign(key, &p, Some(PIN));
    let call = out.signs();
    assert_eq!(call.len(), 1);
    assert_eq!(call[0].pin.as_deref(), Some(PIN), "the key store gets it");
    let mut everything = format!("{:?}{}", out.ui, out.frames_json());

    let out = h.signed_ok(call[0].tag, &p, SHA256, ECDSA);
    everything.push_str(&format!("{:?}{}", out.ui, out.frames_json()));
    assert!(
        !everything.contains(PIN),
        "neither the page nor the window is sent the PIN"
    );
}

#[test]
fn a_wrong_pin_does_not_leak_into_the_window_commands() {
    let (mut h, _, _, tag) = ready();
    let out = h.signed_err(tag, KeystoreError::WrongPin);
    assert!(!format!("{:?}", out.ui).contains(PIN));
}

#[test]
fn a_locked_pin_is_shown_in_the_window_and_the_caller_hears_only_when_it_closes() {
    let (mut h, _, key, tag) = ready();
    let out = h.signed_err(tag, KeystoreError::PinLocked);
    assert!(out.frames.is_empty());
    assert!(matches!(
        out.failures().as_slice(),
        [Failure::PinLocked { .. }]
    ));
    assert!(
        h.state
            .borrow()
            .errors
            .iter()
            .any(|e| e.code == "PinLocked"),
        "the diagnostics store keeps the code"
    );

    // The person closes the window: the last visible blocker is reported.
    let out = h.ui_out(UiEvent::Cancel {
        key,
        code: ErrorCode::PinLocked,
    });
    assert_eq!(out.only_error(), ("s1".to_owned(), ErrorCode::PinLocked));
}

#[test]
fn a_locked_pin_returns_to_selecting_so_another_certificate_can_be_chosen() {
    let (mut h, p, key, tag) = ready();
    h.signed_err(tag, KeystoreError::PinLocked);
    // Still open, and the same certificate cannot simply be signed again.
    let out = h.press_sign(key, &p, Some(PIN));
    assert!(out.signs().is_empty());
}

#[test]
fn cancelling_the_os_dialog_returns_to_ready_silently() {
    let (mut h, p, key, tag) = ready();
    let out = h.signed_err(tag, KeystoreError::Cancelled);
    assert!(out.frames.is_empty());
    assert!(
        out.failures().is_empty(),
        "no message for the person's own cancel"
    );
    assert_eq!(h.press_sign(key, &p, None).signs().len(), 1);
}

#[test]
fn token_removed_is_shown_in_the_window_first() {
    let (mut h, _, key, tag) = ready();
    let out = h.signed_err(tag, KeystoreError::TokenRemoved);
    assert!(out.frames.is_empty());
    assert!(matches!(out.failures().as_slice(), [Failure::TokenRemoved]));

    let out = h.ui_out(UiEvent::Cancel {
        key,
        code: ErrorCode::TokenRemoved,
    });
    assert_eq!(out.only_error().1, ErrorCode::TokenRemoved);
}

#[test]
fn a_missing_key_is_certificate_unavailable_in_the_window() {
    let (mut h, _, _, tag) = ready();
    let out = h.signed_err(tag, KeystoreError::NotFound);
    assert!(matches!(
        out.failures().as_slice(),
        [Failure::CertificateUnavailable]
    ));
    assert!(out.frames.is_empty());
}

#[test]
fn an_unsupported_algorithm_is_shown_in_the_window() {
    let (mut h, _, _, tag) = ready();
    let out = h.signed_err(tag, KeystoreError::Unsupported("PSS".to_owned()));
    assert!(matches!(
        out.failures().as_slice(),
        [Failure::UnsupportedAlgorithm { .. }]
    ));
    assert!(out.frames.is_empty());
}

#[test]
fn a_driver_failure_is_shown_recorded_and_can_be_retried() {
    let (mut h, p, key, tag) = ready();
    let native = KeystoreError::Native {
        api: "C_Sign",
        code: 0x30,
        message: "device error".to_owned(),
    };
    let out = h.signed_err(tag, native);
    assert!(out.frames.is_empty());
    assert!(matches!(
        out.failures().as_slice(),
        [Failure::DriverFailure { .. }]
    ));
    assert_eq!(h.state.borrow().errors.len(), 1);
    assert_eq!(h.state.borrow().errors[0].code, "DriverFailure");
    assert_eq!(
        h.press_sign(key, &p, None).signs().len(),
        1,
        "retry from Ready"
    );
}

#[test]
fn a_reply_for_an_unknown_tag_is_ignored() {
    let (mut h, _, _, tag) = ready();
    let out = h.signed_err(tag + 77, KeystoreError::WrongPin);
    assert!(out.is_silent());
}
