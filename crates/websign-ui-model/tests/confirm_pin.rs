//! ux §4.6 and SPEC §2.2: the PIN area in every state.

mod common;

use common::*;
use websign_protocol::ErrorCode;
use websign_ui_model::certs::{CertCandidate, DeviceLabel, DisabledReason, PinMode, RowStatus};
use websign_ui_model::confirm::port::Failure;
use websign_ui_model::confirm::view::{FooterHint, PinBlock, PinError, PrimaryButton};
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
fn a_token_key_shows_our_field_with_the_token_limits_and_no_error() {
    let view = ready_with(app_pin_token(Some((4, 16)))).view();
    assert_eq!(
        view.pin,
        PinBlock::Field {
            card: false,
            length: Some((4, 16)),
            valid: false,
            error: None,
        }
    );
}

#[test]
fn a_card_in_a_reader_asks_for_a_card_pin() {
    let mut c = app_pin_token(None);
    c.device = Some(DeviceLabel::CardInReader {
        reader: "Identiv uTrust 2700 R".to_owned(),
    });
    let PinBlock::Field { card, .. } = ready_with(c).view().pin else {
        panic!("expected our PIN field");
    };
    assert!(card);
}

#[test]
fn sign_is_enabled_only_while_the_typed_length_is_within_the_limits() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    let table = [
        (0, false),
        (3, false),
        (4, true),
        (10, true),
        (16, true),
        (17, false),
    ];
    for (len, valid) in table {
        w.input(UserInput::PinLength(len));
        let view = w.view();
        assert_eq!(view.footer.primary_enabled, valid, "length {len}");
        let PinBlock::Field {
            valid: field_valid, ..
        } = view.pin
        else {
            panic!("expected our PIN field");
        };
        assert_eq!(field_valid, valid, "length {len}");
    }
}

#[test]
fn enter_and_click_with_an_invalid_length_do_nothing() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(3));
    assert_eq!(w.input(UserInput::Enter), vec![]);
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.state(), ConfirmState::Ready);
}

#[test]
fn enter_in_the_field_signs_when_armed_and_valid() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(6));
    assert_eq!(w.input(UserInput::Enter), vec![sign_intent()]);
    assert_eq!(w.state(), ConfirmState::Signing);
}

#[test]
fn the_button_click_signs_when_the_length_is_valid() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(4));
    assert_eq!(w.click(), vec![sign_intent()]);
}

#[test]
fn without_stated_limits_any_typed_pin_is_valid_but_an_empty_one_is_not() {
    // SPEC: "PIN length valid when the field is shown"; limits are optional.
    let mut w = ready_with(app_pin_token(None));
    assert_eq!(w.input(UserInput::Enter), vec![]);
    w.input(UserInput::PinLength(6));
    assert_eq!(w.input(UserInput::Enter), vec![sign_intent()]);
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
    let locked = list
        .disabled
        .iter()
        .find(|row| row.candidate.fingerprint == fp(1))
        .expect("the locked certificate moves to Can't sign");
    assert_eq!(
        locked.status,
        RowStatus::Disabled(DisabledReason::PinLocked)
    );
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

#[test]
fn a_system_pin_shows_the_os_prompt_hint_before_and_after_the_click() {
    let mut w = ready_remembered(pair(), context());
    let view = w.view();
    assert_eq!(view.pin, PinBlock::OsPrompt { now: false });
    assert_eq!(view.footer.hint, FooterHint::OsPinPrompt);
    w.click();
    w.signing();
    assert_eq!(w.view().pin, PinBlock::OsPrompt { now: true });
}

#[test]
fn a_pin_pad_shows_the_keypad_message_before_and_after_the_click() {
    let mut w = ready_remembered(vec![pin_token(1, "Ana", PinMode::PinPad)], context());
    assert_eq!(w.view().pin, PinBlock::PinPad { now: false });
    assert_eq!(w.click(), vec![sign_intent()]);
    w.signing();
    assert_eq!(w.view().pin, PinBlock::PinPad { now: true });
}

#[test]
fn an_unlocked_token_needs_no_pin_and_signs_directly() {
    let mut w = ready_remembered(vec![pin_token(1, "Ana", PinMode::Unlocked)], context());
    assert_eq!(w.view().pin, PinBlock::Unlocked);
    assert_eq!(w.input(UserInput::Enter), vec![sign_intent()]);
}

#[test]
fn cancel_is_disabled_only_while_the_os_or_the_pin_pad_is_signing() {
    let cases = [
        (pin_token(1, "Ana", PinMode::System), false),
        (pin_token(1, "Ana", PinMode::PinPad), false),
        (pin_token(1, "Ana", app_pin(Some((4, 16)))), true),
        (pin_token(1, "Ana", PinMode::Unlocked), true),
    ];
    for (candidate, cancel_enabled) in cases {
        let pin = candidate.pin;
        let mut w = ready_remembered(vec![candidate], context());
        assert!(w.view().footer.cancel_enabled, "{pin:?} before the click");
        w.input(UserInput::PinLength(6));
        w.click();
        w.input(UserInput::Enter);
        w.signing();
        assert_eq!(w.state(), ConfirmState::Signing, "{pin:?}");
        let view = w.view();
        assert_eq!(view.footer.cancel_enabled, cancel_enabled, "{pin:?}");
        assert_eq!(view.footer.primary, PrimaryButton::Signing, "{pin:?}");
    }
}

#[test]
fn a_removed_token_during_signing_is_an_error_with_a_banner() {
    let mut w = ready_with(app_pin_token(Some((4, 16))));
    w.input(UserInput::PinLength(6));
    w.input(UserInput::Enter);
    w.signing().fail(Failure::TokenRemoved);
    assert_eq!(
        w.state(),
        ConfirmState::Error {
            code: ErrorCode::TokenRemoved
        }
    );
    assert_eq!(w.view().banner, Some(Failure::TokenRemoved));
}
