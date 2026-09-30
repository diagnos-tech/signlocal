//! SPEC §2.2 and ux §15: failures while signing, their banners and how the
//! person recovers (try again, another path, another certificate).

mod common;

use common::*;
use websign_protocol::ErrorCode;
use websign_protocol::types::SignatureAlgorithmName;
use websign_ui_model::certs::PinMode;
use websign_ui_model::confirm::port::Failure;
use websign_ui_model::confirm::view::{PinBlock, PrimaryButton};
use websign_ui_model::confirm::{ConfirmState, Intent, UserInput};

fn signing() -> Window {
    let mut w = ready_remembered(pair(), context());
    assert_eq!(w.click().len(), 1);
    w.signing();
    assert_eq!(w.state(), ConfirmState::Signing);
    w
}

fn sign_via(via: usize) -> Intent {
    Intent::Sign {
        fingerprint: fp(1),
        via,
        remember: false,
    }
}

fn driver_failure(alternate: bool) -> Failure {
    Failure::DriverFailure {
        driver: "eTPKCS11.dll".to_owned(),
        native: "CKR_DEVICE_ERROR (0x00000030)".to_owned(),
        alternate,
    }
}

#[test]
fn failures_other_than_pin_ones_become_error_states_with_their_code() {
    let cases = [
        (Failure::TokenRemoved, ErrorCode::TokenRemoved),
        (
            Failure::DriverFailure {
                driver: "eTPKCS11.dll".to_owned(),
                native: "CKR_DEVICE_ERROR (0x00000030)".to_owned(),
                alternate: false,
            },
            ErrorCode::DriverFailure,
        ),
        (
            Failure::UnsupportedAlgorithm {
                algorithm: SignatureAlgorithmName::Ecdsa,
            },
            ErrorCode::UnsupportedAlgorithm,
        ),
        (
            Failure::CertificateUnavailable,
            ErrorCode::CertificateUnavailable,
        ),
        (
            Failure::Internal {
                detail: "bug".to_owned(),
            },
            ErrorCode::Internal,
        ),
    ];
    for (failure, code) in cases {
        let mut w = signing();
        w.fail(failure.clone());
        assert_eq!(w.state(), ConfirmState::Error { code }, "{failure:?}");
        assert_eq!(w.view().banner, Some(failure.clone()), "{failure:?}");
    }
}

#[test]
fn a_recoverable_error_offers_try_again_and_signs_again() {
    for failure in [driver_failure(false), Failure::TokenRemoved] {
        let mut w = signing();
        w.fail(failure.clone());
        w.wait(1_000);
        assert_eq!(w.view().footer.primary, PrimaryButton::Retry, "{failure:?}");
        assert_eq!(w.click(), vec![sign_via(0)], "{failure:?}");
        assert_eq!(w.state(), ConfirmState::Signing);
    }
}

#[test]
fn an_internal_error_offers_no_try_again() {
    let mut w = signing();
    w.fail(Failure::Internal {
        detail: "bug".to_owned(),
    });
    w.wait(1_000);
    let footer = w.view().footer;
    assert_eq!(footer.primary, PrimaryButton::Sign);
    assert!(!footer.primary_enabled);
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.input(UserInput::Enter), vec![]);
}

#[test]
fn an_error_that_cannot_be_retried_disables_the_button() {
    for failure in [
        Failure::CertificateUnavailable,
        Failure::UnsupportedAlgorithm {
            algorithm: SignatureAlgorithmName::RsaPss,
        },
    ] {
        let mut w = signing();
        w.fail(failure.clone());
        w.wait(1_000);
        assert!(!w.view().footer.primary_enabled, "{failure:?}");
        assert_eq!(w.click(), vec![], "{failure:?}");
    }
}

#[test]
fn a_driver_failure_with_an_alternate_path_can_retry_through_the_driver() {
    // `via` 0 is the primary path; the first alternate is index 1. A retry
    // after the alternate failed repeats the path that failed last.
    let mut candidates = pair();
    candidates[0].alternates = vec![driver_path("eTPKCS11.dll", PinMode::Unlocked)];
    let mut w = ready_remembered(candidates, context());
    w.click();
    w.signing().fail(driver_failure(true));
    w.wait(1_000);
    assert_eq!(w.input(UserInput::UseAlternatePath), vec![sign_via(1)]);
    assert_eq!(w.state(), ConfirmState::Signing);
    w.fail(driver_failure(false));
    assert_eq!(w.click(), vec![sign_via(1)]);
}

#[test]
fn a_driver_path_that_needs_our_pin_shows_the_field_before_signing() {
    // The OS store asked through its own dialog, so no PIN was typed: the
    // driver's C_Login needs one from our field (ux §4.6, §5.11).
    let mut candidates = pair();
    candidates[0].alternates = vec![driver_path("eTPKCS11.dll", app_pin(Some((4, 8))))];
    let mut w = ready_remembered(candidates, context());
    w.click();
    w.signing().fail(driver_failure(true));
    w.wait(1_000);
    assert_eq!(w.input(UserInput::UseAlternatePath), vec![]);
    assert_eq!(w.state(), ConfirmState::Ready);
    assert!(matches!(
        w.view().pin,
        PinBlock::Field {
            length: Some((4, 8)),
            valid: false,
            ..
        }
    ));
    assert_eq!(w.view().banner, None);
    w.input(UserInput::PinLength(4));
    assert_eq!(w.click(), vec![], "re-armed: the field is new content");
    w.wait(1_000);
    assert_eq!(w.click(), vec![sign_via(1)]);
}

#[test]
fn the_alternate_path_is_offered_only_when_the_failure_says_so() {
    let mut w = signing();
    w.fail(driver_failure(false));
    w.wait(1_000);
    assert_eq!(w.input(UserInput::UseAlternatePath), vec![]);
}

#[test]
fn another_certificate_can_be_chosen_after_an_error() {
    let mut w = signing();
    w.fail(Failure::TokenRemoved);
    w.wait(1_000);
    assert_eq!(
        w.input(UserInput::Select(fp(2))),
        vec![Intent::Selected(fp(2))]
    );
    assert_eq!(w.state(), ConfirmState::Choosing);
}

#[test]
fn an_error_is_cleared_by_the_next_selection() {
    let mut w = signing();
    w.fail(Failure::TokenRemoved);
    w.wait(1_000);
    w.input(UserInput::Select(fp(2)));
    assert_eq!(w.view().banner, None);
}

#[test]
fn rescan_and_diagnostics_are_forwarded_when_armed() {
    let mut w = ready_remembered(pair(), context());
    assert_eq!(w.input(UserInput::Rescan), vec![Intent::Rescan]);
    assert!(diagnostics_intent(&w.input(UserInput::OpenDiagnostics)));
}

fn diagnostics_intent(intents: &[Intent]) -> bool {
    matches!(intents, [Intent::OpenDiagnostics(_)])
}
