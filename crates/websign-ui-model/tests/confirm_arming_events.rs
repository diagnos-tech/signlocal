//! ux §4.7 and SPEC §2.2: what re-arms the primary button, what is ignored
//! while unarmed, and the Esc / Enter rules.

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
fn focus_loss_disarms_and_regaining_focus_re_arms_for_600_ms() {
    let mut w = ready_remembered(pair(), context());
    assert!(w.view().footer.primary_enabled);

    w.input(UserInput::Focus(false));
    assert!(!w.view().footer.primary_enabled);
    w.wait(5_000);
    assert_eq!(w.click(), vec![], "still unfocused");

    w.input(UserInput::Focus(true));
    w.wait(599);
    assert_eq!(w.click(), vec![]);
    assert!(!w.view().footer.primary_enabled);
    w.wait(1);
    assert!(w.view().footer.primary_enabled);
    assert_eq!(w.click(), vec![sign(1)]);
}

#[test]
fn a_press_made_before_focus_loss_does_not_click_after_it() {
    let mut w = ready_remembered(pair(), context());
    w.input(UserInput::PrimaryPress);
    w.input(UserInput::Focus(false));
    w.input(UserInput::Focus(true));
    w.wait(2_000);
    assert_eq!(w.input(UserInput::PrimaryRelease), vec![]);
    assert_eq!(w.state(), ConfirmState::Ready);
}

#[test]
fn a_press_before_arming_released_after_arming_does_not_click() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), true));
    w.wait(50).certificates(pair(), context());
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    w.wait(300);
    assert_eq!(w.input(UserInput::PrimaryPress), vec![]);
    w.wait(1_000);
    assert_eq!(w.input(UserInput::PrimaryRelease), vec![]);
    assert_eq!(w.state(), ConfirmState::Ready);
}

#[test]
fn changing_the_certificate_re_arms() {
    let mut w = ready_remembered(pair(), context());
    w.input(UserInput::Select(fp(2)));
    w.digest(fp(2), "1111 2222 3333 4444");
    w.wait(599);
    assert_eq!(w.click(), vec![]);
    w.wait(1);
    assert_eq!(w.click(), vec![sign(2)]);
}

#[test]
fn a_new_digest_for_the_same_certificate_re_arms() {
    // ux §4.7: "the digest changes" re-arms. SPEC lists only Choosing ->
    // Ready; a second DigestReady while Ready is assumed to behave the same.
    let mut w = ready_remembered(pair(), context());
    w.digest(fp(1), "AAAA BBBB CCCC DDDD");
    w.wait(300);
    assert_eq!(w.click(), vec![]);
    w.wait(300);
    assert_eq!(w.click(), vec![sign(1)]);
}

#[test]
fn a_token_inserted_while_ready_re_arms_and_keeps_the_selection() {
    // ux §4.8: "Token inserted ... the button re-arms."
    let mut w = ready_remembered(pair(), context());
    let mut all = pair();
    all.push(candidate(3, "Carla"));
    w.certificates(all, context());
    assert_eq!(w.state(), ConfirmState::Ready);
    assert_eq!(w.view().selected, Some(fp(1)));
    w.wait(300);
    assert_eq!(w.click(), vec![]);
    w.wait(300);
    assert_eq!(w.click(), vec![sign(1)]);
}

#[test]
fn a_queue_update_does_not_re_arm() {
    let mut w = ready_remembered(pair(), context());
    w.apply(UiCommand::Queue {
        key: KEY,
        position: (1, 3),
    });
    assert!(w.view().footer.primary_enabled);
    assert_eq!(w.click(), vec![sign(1)]);
}

#[test]
fn a_new_open_re_arms_without_another_focus_event() {
    // SPEC §2.2: "a `Queue` update does not re-arm, a new `Open` does".
    let mut w = ready_remembered(pair(), context());
    assert_eq!(w.click(), vec![sign(1)]);
    w.signing()
        .finish(websign_ui_model::confirm::port::Finish::Signed);
    w.wait(900);
    w.tick();

    let next = websign_ui_model::confirm::port::RequestKey(2);
    w.key = next;
    w.apply(UiCommand::Open(request(next, sign_mode(), false)));
    w.wait(10).certificates(pair(), context());
    w.wait(300);
    assert_eq!(w.state(), ConfirmState::Choosing);
    assert_eq!(w.click(), vec![], "600 ms have not passed since Open");
    w.wait(1_000);
    assert_eq!(w.click(), vec![Intent::Continue(fp(1))]);
}

#[test]
fn keys_and_clicks_are_discarded_while_unarmed() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), true));
    w.wait(10).certificates(pair(), context());
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    w.wait(100);
    assert_eq!(w.state(), ConfirmState::Ready);
    assert_eq!(w.input(UserInput::Enter), vec![]);
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.input(UserInput::Rescan), vec![]);
    assert_eq!(w.state(), ConfirmState::Ready);
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
fn cancel_and_close_buttons_send_the_same_code_when_armed() {
    for input in [UserInput::CancelButton, UserInput::CloseButton] {
        let mut w = ready_remembered(pair(), context());
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
