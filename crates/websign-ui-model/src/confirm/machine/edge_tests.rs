use std::time::Duration;

use websign_protocol::ErrorCode;

use super::rig::{KEY, Rig, SIGN};
use super::*;
use crate::confirm::port::{Failure, Finish, OpenRequest, RequestKey};
use crate::confirm::view::{FooterHint, PrimaryButton, RememberBox};
use crate::fixtures::fingerprint;

fn signing_rig() -> Rig {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    rig.digest_ready(1);
    rig
}

#[test]
fn site_cancelled_holds_one_and_a_half_seconds() {
    let mut rig = signing_rig();
    rig.finish(Finish::SiteCancelled);
    assert_eq!(*rig.model.state(), ConfirmState::SiteCancelled);
    rig.wait(1499);
    rig.model.tick(rig.now());
    assert_eq!(*rig.model.state(), ConfirmState::SiteCancelled);
    rig.wait(1);
    rig.model.tick(rig.now());
    assert_eq!(*rig.model.state(), ConfirmState::Idle);
}

#[test]
fn timeout_and_abort() {
    let mut rig = signing_rig();
    rig.finish(Finish::Timeout);
    assert_eq!(*rig.model.state(), ConfirmState::Timeout);
    assert!(rig.input(UserInput::Escape).is_empty());
    assert_eq!(*rig.model.state(), ConfirmState::Idle);
    let mut rig = signing_rig();
    rig.finish(Finish::Aborted);
    assert_eq!(*rig.model.state(), ConfirmState::Idle);
}

#[test]
fn countdown_shows_in_the_last_thirty_seconds() {
    let mut rig = signing_rig();
    assert_eq!(rig.view().footer.hint, FooterHint::OsPinPrompt);
    rig.wait(269_000);
    assert_eq!(rig.view().footer.hint, FooterHint::OsPinPrompt);
    let start = rig.now();
    assert_eq!(
        rig.model.next_deadline(start),
        Some(start + Duration::from_secs(1))
    );
    rig.wait(1000);
    assert_eq!(
        rig.view().footer.hint,
        FooterHint::ExpiresIn { seconds: 30 }
    );
    rig.wait(1500);
    assert_eq!(
        rig.view().footer.hint,
        FooterHint::ExpiresIn { seconds: 29 }
    );
    let now = rig.now();
    assert_eq!(
        rig.model.next_deadline(now),
        Some(now + Duration::from_millis(500))
    );
}

#[test]
fn arming_and_skeleton_are_deadlines() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    let t0 = rig.now();
    assert_eq!(
        rig.model.next_deadline(t0),
        Some(t0 + Duration::from_millis(150))
    );
    rig.wait(150);
    assert_eq!(
        rig.model.next_deadline(rig.now()),
        Some(t0 + Duration::from_millis(600))
    );
    rig.wait(500);
    assert!(
        rig.model
            .next_deadline(rig.now())
            .is_some_and(|at| at > rig.now())
    );
    assert_eq!(ConfirmModel::new().next_deadline(t0), None);
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
    assert!(rig.input(UserInput::CloseButton).is_empty(), "sent once");
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
