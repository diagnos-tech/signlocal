//! SPEC §2.2 and ux §4.8, §4.11: how a request ends (success hold, site
//! gave up, timeout, errors and their recovery).

mod common;

use common::*;
use websign_protocol::ErrorCode;
use websign_protocol::types::SignatureAlgorithmName;
use websign_ui_model::confirm::port::{Failure, Finish, RequestKey, UiCommand};
use websign_ui_model::confirm::view::PrimaryButton;
use websign_ui_model::confirm::{ConfirmState, Intent, UserInput};

fn signing() -> Window {
    let mut w = ready_remembered(pair(), context());
    assert_eq!(w.click().len(), 1);
    w.signing();
    assert_eq!(w.state(), ConfirmState::Signing);
    w
}

#[test]
fn a_signature_shows_success_for_900_ms_then_the_window_is_idle() {
    let mut w = signing();
    w.finish(Finish::Signed);
    assert_eq!(w.state(), ConfirmState::Success);
    w.wait(899);
    assert_eq!(w.tick(), vec![]);
    assert_eq!(w.state(), ConfirmState::Success);
    w.wait(1);
    w.tick();
    assert_eq!(w.state(), ConfirmState::Idle);
}

#[test]
fn ticking_early_never_ends_the_success_hold() {
    let mut w = signing();
    w.finish(Finish::Signed);
    for _ in 0..8 {
        w.wait(100);
        w.tick();
        assert_eq!(w.state(), ConfirmState::Success);
    }
}

#[test]
fn the_next_request_takes_over_during_the_success_hold() {
    let mut w = signing();
    w.finish(Finish::Signed);
    w.wait(300);
    let next = RequestKey(2);
    w.key = next;
    w.apply(UiCommand::Open(request(next, sign_mode(), false)));
    assert_eq!(w.state(), ConfirmState::LoadingCerts);
    w.wait(2_000);
    w.tick();
    assert_eq!(
        w.state(),
        ConfirmState::LoadingCerts,
        "the old hold is gone"
    );
}

#[test]
fn success_is_final_for_input() {
    let mut w = signing();
    w.finish(Finish::Signed);
    w.wait(100);
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.state(), ConfirmState::Success);
}

#[test]
fn a_site_that_gives_up_shows_the_notice_for_1500_ms() {
    let mut w = ready_remembered(pair(), context());
    w.finish(Finish::SiteCancelled);
    assert_eq!(w.state(), ConfirmState::SiteCancelled);
    w.wait(1_499);
    w.tick();
    assert_eq!(w.state(), ConfirmState::SiteCancelled);
    w.wait(1);
    w.tick();
    assert_eq!(w.state(), ConfirmState::Idle);
}

#[test]
fn a_site_can_give_up_in_any_state() {
    // Loading, choosing, ready, signing.
    let mut loading = Window::new();
    loading.open(request(KEY, sign_mode(), false));
    loading.finish(Finish::SiteCancelled);
    assert_eq!(loading.state(), ConfirmState::SiteCancelled);

    let mut choosing = Window::new();
    choosing.open(request(KEY, sign_mode(), false));
    choosing.wait(10).certificates(pair(), context());
    choosing.finish(Finish::SiteCancelled);
    assert_eq!(choosing.state(), ConfirmState::SiteCancelled);

    let mut in_signing = signing();
    in_signing.finish(Finish::SiteCancelled);
    assert_eq!(in_signing.state(), ConfirmState::SiteCancelled);
}

#[test]
fn a_timeout_finish_shows_the_timeout_state() {
    let mut w = ready_remembered(pair(), context());
    w.finish(Finish::Timeout);
    assert_eq!(w.state(), ConfirmState::Timeout);
}

#[test]
fn escape_after_a_finish_notice_reports_a_plain_cancel() {
    let mut w = ready_remembered(pair(), context());
    w.finish(Finish::SiteCancelled);
    assert_eq!(
        w.input(UserInput::Escape),
        vec![Intent::Cancel(ErrorCode::UserCancelled)]
    );
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
    let failure = Failure::DriverFailure {
        driver: "eTPKCS11.dll".to_owned(),
        native: "CKR_DEVICE_ERROR (0x00000030)".to_owned(),
        alternate: false,
    };
    let mut w = signing();
    w.fail(failure);
    w.wait(1_000);
    assert_eq!(w.view().footer.primary, PrimaryButton::Retry);
    assert_eq!(
        w.click(),
        vec![Intent::Sign {
            fingerprint: fp(1),
            via: 0,
            remember: false
        }]
    );
    assert_eq!(w.state(), ConfirmState::Signing);
}

#[test]
fn a_driver_failure_with_an_alternate_path_can_retry_through_the_driver() {
    // SPEC: `via` 0 is the primary path; the alternate is index 1.
    let failure = Failure::DriverFailure {
        driver: "Windows".to_owned(),
        native: "NTE_FAIL".to_owned(),
        alternate: true,
    };
    let mut candidates = pair();
    candidates[0].alternates = vec![driver("eTPKCS11.dll")];
    let mut w = ready_remembered(candidates, context());
    w.click();
    w.signing().fail(failure);
    w.wait(1_000);
    assert_eq!(
        w.input(UserInput::UseAlternatePath),
        vec![Intent::Sign {
            fingerprint: fp(1),
            via: 1,
            remember: false
        }]
    );
    assert_eq!(w.state(), ConfirmState::Signing);
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
