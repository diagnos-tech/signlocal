//! ux §4.7 and SPEC §2.2: what arms and re-arms the primary button, and
//! what is ignored while unarmed.

mod common;

use common::*;
use websign_ui_model::confirm::port::UiCommand;
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
    // ux §4.7: "the digest changes" re-arms, also while already Ready.
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
    // The next request is new content under the pointer: a focused window
    // re-arms on `Open`, while a mere `Queue` update does not.
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
fn a_window_that_never_got_focus_never_arms() {
    // The 600 ms count from visible *and* focused: a window shown behind
    // the browser cannot be approved by a click that only brings it forward.
    let mut w = Window::new();
    w.apply(UiCommand::Open(request(KEY, sign_mode(), true)));
    w.wait(10).certificates(pair(), context());
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    w.wait(5_000);
    assert!(!w.view().footer.primary_enabled);
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.input(UserInput::Enter), vec![]);
    w.input(UserInput::Focus(true));
    assert_eq!(w.click(), vec![], "focus starts the count");
    w.wait(600);
    assert_eq!(w.click(), vec![sign(1)]);
}

#[test]
fn a_token_that_comes_back_re_arms() {
    let mut w = ready_remembered(pair(), context());
    let mut gone = pair();
    gone[0].removed = true;
    w.certificates(gone, context());
    assert!(!w.view().footer.primary_enabled);
    w.wait(1_000).certificates(pair(), context());
    w.wait(599);
    assert_eq!(w.click(), vec![]);
    w.wait(1);
    assert_eq!(w.click(), vec![sign(1)]);
}
