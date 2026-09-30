//! Errors and their recovery, queue and stale commands, the remember box.

use websign_protocol::ErrorCode;

use super::rig::{KEY, Rig, SIGN};
use super::*;
use crate::confirm::port::{Failure, Finish, OpenRequest, RequestKey};
use crate::confirm::view::{PrimaryButton, RememberBox};
use crate::fixtures::fingerprint;

fn signing_rig() -> Rig {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    rig.digest_ready(1);
    rig
}

#[test]
fn recoverable_error_offers_retry_and_alternate_path() {
    let mut rig = signing_rig();
    rig.click();
    rig.apply(UiCommand::Failed {
        key: KEY,
        failure: Failure::DriverFailure {
            driver: "SAC".into(),
            native: "CKR_DEVICE_ERROR (0x00000030)".into(),
            alternate: true,
        },
    });
    assert_eq!(
        *rig.model.state(),
        ConfirmState::Error {
            code: ErrorCode::DriverFailure
        }
    );
    rig.wait(700);
    let view = rig.view();
    assert_eq!(view.footer.primary, PrimaryButton::Retry);
    assert!(view.footer.primary_enabled);
    assert!(view.banner.is_some());
    assert_eq!(
        rig.input(UserInput::UseAlternatePath),
        [Intent::Sign {
            fingerprint: fingerprint(1),
            via: 1,
            remember: false
        }]
    );
}

#[test]
fn retry_signs_again_and_cancel_keeps_user_cancelled() {
    let mut rig = signing_rig();
    rig.click();
    rig.apply(UiCommand::Failed {
        key: KEY,
        failure: Failure::TokenRemoved,
    });
    assert_eq!(rig.click().len(), 1);
    rig.apply(UiCommand::Failed {
        key: KEY,
        failure: Failure::TokenRemoved,
    });
    assert_eq!(
        rig.input(UserInput::Escape),
        [Intent::Cancel(ErrorCode::UserCancelled)]
    );
}

#[test]
fn unavailable_certificate_cancels_with_its_code() {
    let mut rig = signing_rig();
    rig.click();
    rig.apply(UiCommand::Failed {
        key: KEY,
        failure: Failure::CertificateUnavailable,
    });
    assert_eq!(rig.view().footer.primary, PrimaryButton::Sign);
    assert!(!rig.view().footer.primary_enabled);
    assert_eq!(
        rig.input(UserInput::CloseButton),
        [Intent::Cancel(ErrorCode::CertificateUnavailable)]
    );
    assert_eq!(
        rig.input(UserInput::Escape),
        [Intent::Cancel(ErrorCode::CertificateUnavailable)],
        "a repeated close is never swallowed"
    );
}

#[test]
fn queue_update_does_not_rearm_but_open_does() {
    let mut rig = signing_rig();
    rig.wait(700);
    rig.apply(UiCommand::Queue {
        key: KEY,
        position: (1, 3),
    });
    assert!(rig.view().footer.primary_enabled);
    assert_eq!(rig.view().header.queue, Some((1, 3)));
    let mut next = OpenRequest {
        key: RequestKey(2),
        ..super::rig::request(SIGN, true)
    };
    next.position = (2, 3);
    rig.apply(UiCommand::Open(next));
    assert_eq!(*rig.model.state(), ConfirmState::LoadingCerts);
    assert!(!rig.model.arming.is_armed(rig.now()));
}

#[test]
fn commands_for_other_requests_are_ignored() {
    let mut rig = signing_rig();
    rig.apply(UiCommand::Finished {
        key: RequestKey(99),
        finish: Finish::Signed,
    });
    assert_eq!(*rig.model.state(), ConfirmState::Ready);
    rig.apply(UiCommand::Hide);
    assert_eq!(*rig.model.state(), ConfirmState::Idle);
}

#[test]
fn remember_box_variants() {
    let mut rig = Rig::new();
    let mut request = super::rig::request(SIGN, false);
    request.can_remember = false;
    rig.apply(UiCommand::Open(request));
    assert_eq!(rig.view().remember, RememberBox::Disabled);
    rig.wait(700);
    rig.input(UserInput::Remember(true));
    assert_eq!(rig.view().remember, RememberBox::Disabled);
}

#[test]
fn rescan_from_empty_looks_again() {
    let mut rig = Rig::new();
    rig.open(SIGN, false, Vec::new());
    rig.wait(700);
    assert_eq!(rig.input(UserInput::Rescan), [Intent::Rescan]);
    assert_eq!(*rig.model.state(), ConfirmState::LoadingCerts);
    assert_eq!(
        rig.input(UserInput::OpenDiagnostics),
        [Intent::OpenDiagnostics(None)]
    );
}

#[test]
fn unsupported_algorithm_marks_the_row_incompatible() {
    let mut rig = signing_rig();
    rig.click();
    rig.apply(UiCommand::Failed {
        key: KEY,
        failure: Failure::UnsupportedAlgorithm {
            algorithm: websign_protocol::types::SignatureAlgorithmName::RsaPss,
        },
    });
    let list = rig.view().list.expect("list");
    assert_eq!(
        list.usable[0].status,
        crate::certs::RowStatus::Disabled(crate::certs::DisabledReason::Incompatible)
    );
}
