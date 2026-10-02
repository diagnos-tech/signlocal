//! ux §4.6 and SPEC §2.2: a wrong PIN and a locked PIN.

mod common;

use common::*;
use websign_protocol::ErrorCode;
use websign_ui_model::certs::{CertCandidate, DisabledReason, RowStatus};
use websign_ui_model::confirm::port::Failure;
use websign_ui_model::confirm::view::{PinBlock, PinError};
use websign_ui_model::confirm::{ConfirmState, Intent, UserInput};

fn app_pin_token(length: Option<(u32, u32)>) -> CertCandidate {
    pin_token(1, "Ana", app_pin(length))
}

fn ready_with(pin: CertCandidate) -> Window {
    ready_remembered(vec![pin, candidate(2, "Bia")], context())
}

fn sign_intent() -> Intent {
    Intent::Sign {
        fingerprint: fp(1),
        via: 0,
        remember: false,
    }
}

fn incorrect(count_low: bool, final_try: bool) -> Failure {
    Failure::PinIncorrect {
        count_low,
        final_try,
    }
}

#[test]
fn a_wrong_pin_shows_the_incorrect_message_and_returns_to_typing() {
    let cases = [
        (incorrect(false, false), PinError::Incorrect),
        (incorrect(true, false), PinError::IncorrectLow),
        (incorrect(false, true), PinError::IncorrectFinal),
        (incorrect(true, true), PinError::IncorrectFinal),
    ];
    for (failure, expected) in cases {
        let mut w = ready_with(app_pin_token(Some((4, 16))));
        w.input(UserInput::PinLength(6));
        w.input(UserInput::Enter);
        w.signing().fail(failure.clone());
        assert_eq!(w.state(), ConfirmState::PinError, "{failure:?}");
        let PinBlock::Field { error, valid, .. } = w.view().pin else {
            panic!("the field stays for a retry");
        };
        assert_eq!(error, Some(expected), "{failure:?}");
        assert!(!valid, "the field was emptied");
    }
}

#[test]
fn after_a_wrong_pin_the_person_must_type_again_before_signing() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(6));
    w.input(UserInput::Enter);
    w.signing().fail(incorrect(false, false));
    w.wait(2_000);

    assert_eq!(w.input(UserInput::Enter), vec![]);
    assert!(!w.view().footer.primary_enabled);

    w.input(UserInput::PinLength(6));
    assert_eq!(w.input(UserInput::Enter), vec![sign_intent()]);
    assert_eq!(w.state(), ConfirmState::Signing);
}

#[test]
fn the_pin_error_clears_when_signing_again() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(6));
    w.input(UserInput::Enter);
    w.signing().fail(incorrect(false, false));
    w.wait(2_000);
    w.input(UserInput::PinLength(6));
    w.input(UserInput::Enter);
    let PinBlock::Field { error, .. } = w.view().pin else {
        panic!("field expected");
    };
    assert_eq!(error, None);
}

#[test]
fn a_wrong_pin_is_not_a_banner() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(6));
    w.input(UserInput::Enter);
    w.signing().fail(incorrect(false, false));
    assert_eq!(w.view().banner, None);
}

#[test]
fn a_locked_pin_replaces_the_field_disables_the_row_and_the_button() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(6));
    w.input(UserInput::Enter);
    w.signing().fail(Failure::PinLocked {
        tool: Some("SafeNet Authentication Client".to_owned()),
        issuer: "AC SOLUTI".to_owned(),
    });
    assert_eq!(w.state(), ConfirmState::PinLocked);
    let view = w.view();
    assert_eq!(view.pin, PinBlock::Locked);
    assert!(!view.footer.primary_enabled);
    let list = view.list.expect("the list stays on screen");
    // Disabled in place: nothing moves under the pointer (SPEC §2.2).
    assert_eq!(usable(&list)[0], fp(1));
    assert!(disabled(&list).is_empty());
    assert_eq!(
        status(&list, fp(1)),
        RowStatus::Disabled(DisabledReason::PinLocked)
    );
    assert_eq!(view.selected, Some(fp(1)));
    assert_eq!(w.input(UserInput::Enter), vec![]);
}

#[test]
fn cancelling_a_locked_pin_reports_pin_locked() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(6));
    w.input(UserInput::Enter);
    w.signing().fail(Failure::PinLocked {
        tool: None,
        issuer: "AC".to_owned(),
    });
    for input in [
        UserInput::Escape,
        UserInput::CancelButton,
        UserInput::CloseButton,
    ] {
        assert_eq!(
            w.input(input.clone()),
            vec![Intent::Cancel(ErrorCode::PinLocked)],
            "{input:?}"
        );
    }
}

#[test]
fn after_a_lockout_another_certificate_can_be_chosen() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(6));
    w.input(UserInput::Enter);
    w.signing().fail(Failure::PinLocked {
        tool: None,
        issuer: "AC".to_owned(),
    });
    w.wait(1_000);
    assert_eq!(
        w.input(UserInput::Select(fp(2))),
        vec![Intent::Selected(fp(2))]
    );
    assert_eq!(w.state(), ConfirmState::Choosing);
    assert_eq!(w.view().selected, Some(fp(2)));
    assert_ne!(w.view().pin, PinBlock::Locked);
}
