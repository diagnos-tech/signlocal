//! ux §4.7, §4.9 and SPEC §2.2: leaving (Esc, Cancel, close) always works,
//! armed or not; Enter only ever signs, never from the list.

mod common;

use common::*;
use websign_protocol::ErrorCode;
use websign_ui_model::certs::PinMode;
use websign_ui_model::confirm::port::{Failure, UiCommand};
use websign_ui_model::confirm::{ConfirmState, Intent, UserInput};

fn sign(fingerprint: u8) -> Intent {
    Intent::Sign {
        fingerprint: fp(fingerprint),
        via: 0,
        remember: false,
    }
}

#[test]
fn enter_signs_only_once_armed() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), true));
    w.wait(10).certificates(pair(), context());
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    w.wait(599);
    assert_eq!(w.input(UserInput::Enter), vec![]);
    w.wait(1);
    assert_eq!(w.input(UserInput::Enter), vec![sign(1)]);
}

#[test]
fn escape_cancels_even_while_unarmed() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), true));
    w.wait(10).certificates(pair(), context());
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    w.wait(50);
    assert_eq!(
        w.input(UserInput::Escape),
        vec![Intent::Cancel(ErrorCode::UserCancelled)]
    );
}

#[test]
fn escape_cancels_from_loading_and_choosing() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    assert_eq!(
        w.input(UserInput::Escape),
        vec![Intent::Cancel(ErrorCode::UserCancelled)]
    );
    w.wait(10).certificates(pair(), context());
    assert_eq!(
        w.input(UserInput::Escape),
        vec![Intent::Cancel(ErrorCode::UserCancelled)]
    );
}

#[test]
fn cancel_and_close_buttons_work_while_unarmed() {
    for input in [UserInput::CancelButton, UserInput::CloseButton] {
        let mut w = Window::new();
        w.apply(UiCommand::Open(request(KEY, sign_mode(), true)));
        w.wait(10).certificates(pair(), context());
        assert_eq!(
            w.input(input.clone()),
            vec![Intent::Cancel(ErrorCode::UserCancelled)],
            "{input:?}"
        );
    }
}

#[test]
fn escape_with_nothing_on_screen_does_nothing() {
    let mut w = Window::new();
    assert_eq!(w.input(UserInput::Escape), vec![]);
}

#[test]
fn escape_reports_the_blocking_condition_on_screen() {
    // Empty -> NoCertificates.
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(10).certificates(vec![], context());
    assert_eq!(w.state(), ConfirmState::Empty);
    assert_eq!(
        w.input(UserInput::Escape),
        vec![Intent::Cancel(ErrorCode::NoCertificates)]
    );

    // Unavailable certificate -> CertificateUnavailable.
    let mut w = ready_remembered(pair(), context());
    w.click();
    w.signing().fail(Failure::CertificateUnavailable);
    assert_eq!(
        w.input(UserInput::Escape),
        vec![Intent::Cancel(ErrorCode::CertificateUnavailable)]
    );
}

#[test]
fn a_token_pin_keeps_the_same_arming_rules() {
    let token = pin_token(
        1,
        "Ana",
        PinMode::App {
            length: Some((4, 16)),
            count_low: false,
            final_try: false,
            locked: false,
        },
    );
    let mut w = ready_remembered(vec![token], context());
    w.input(UserInput::PinLength(6));
    assert_eq!(w.input(UserInput::Enter), vec![sign(1)]);
}

#[test]
fn every_press_of_escape_sends_a_cancel() {
    let mut w = ready_remembered(pair(), context());
    for _ in 0..2 {
        assert_eq!(
            w.input(UserInput::Escape),
            vec![Intent::Cancel(ErrorCode::UserCancelled)]
        );
    }
}

#[test]
fn escape_is_ignored_while_the_os_asks_for_the_pin() {
    // Neither the OS dialog nor a PIN pad can be aborted from our window.
    let mut w = ready_remembered(pair(), context());
    assert_eq!(w.click(), vec![sign(1)]);
    w.signing();
    assert!(!w.view().footer.cancel_enabled);
    assert_eq!(w.input(UserInput::Escape), vec![]);
    assert_eq!(w.input(UserInput::CancelButton), vec![]);
    assert_eq!(w.state(), ConfirmState::Signing);
}

#[test]
fn escape_during_signing_with_our_pin_field_cancels() {
    let token = pin_token(1, "Ana", app_pin(Some((4, 16))));
    let mut w = ready_remembered(vec![token], context());
    w.input(UserInput::PinLength(6));
    assert_eq!(w.input(UserInput::Enter), vec![sign(1)]);
    w.signing();
    assert!(w.view().footer.cancel_enabled);
    assert_eq!(
        w.input(UserInput::Escape),
        vec![Intent::Cancel(ErrorCode::UserCancelled)]
    );
}
