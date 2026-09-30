//! SPEC §2.2: a remembered caller has its certificate released by the host,
//! so the window goes straight from the list to the digest.

mod common;

use common::*;
use websign_ui_model::confirm::view::{CodeCard, PrimaryButton, RememberBox};
use websign_ui_model::confirm::{ConfirmState, Intent, UserInput};

fn choosing() -> Window {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), true));
    w.wait(100).certificates(pair(), context());
    w.wait(1000);
    w
}

#[test]
fn the_code_card_is_preparing_and_there_is_no_continue_step() {
    let view = choosing().view();
    assert_eq!(view.state, ConfirmState::Choosing);
    assert_eq!(view.code, CodeCard::Preparing { skeleton: false });
    assert_ne!(view.footer.primary, PrimaryButton::Continue);
}

#[test]
fn the_digest_makes_it_ready() {
    let mut w = choosing();
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    assert_eq!(w.state(), ConfirmState::Ready);
    assert_eq!(w.view().footer.primary, PrimaryButton::Sign);
}

#[test]
fn the_header_says_the_caller_is_remembered() {
    let view = choosing().view();
    assert!(view.header.remembered);
    assert_eq!(view.header.caller, caller());
}

#[test]
fn the_remember_box_is_hidden() {
    assert_eq!(choosing().view().remember, RememberBox::Hidden);
}

#[test]
fn a_click_while_preparing_signs_nothing() {
    let mut w = choosing();
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.state(), ConfirmState::Choosing);
}

#[test]
fn a_digest_re_arms_the_button() {
    let mut w = choosing();
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    w.wait(599);
    assert_eq!(w.click(), vec![]);
    w.wait(1);
    // The failed click did not move the clock; 600 ms after the digest:
    assert!(w.view().footer.primary_enabled);
    assert_eq!(
        w.click(),
        vec![Intent::Sign {
            fingerprint: fp(1),
            via: 0,
            remember: false
        }]
    );
}

#[test]
fn changing_the_certificate_asks_for_a_new_digest() {
    let mut w = ready_remembered(pair(), context());
    assert_eq!(
        w.input(UserInput::Select(fp(2))),
        vec![Intent::Selected(fp(2))]
    );
    assert_eq!(w.state(), ConfirmState::Choosing);
    let view = w.view();
    assert_eq!(view.selected, Some(fp(2)));
    assert_eq!(view.code, CodeCard::Preparing { skeleton: false });
}

#[test]
fn the_skeleton_appears_after_150_ms_of_preparing() {
    let mut w = ready_remembered(pair(), context());
    w.input(UserInput::Select(fp(2)));
    assert_eq!(w.view().code, CodeCard::Preparing { skeleton: false });
    w.wait(149);
    assert_eq!(w.view().code, CodeCard::Preparing { skeleton: false });
    w.wait(1);
    assert_eq!(w.view().code, CodeCard::Preparing { skeleton: true });
}

#[test]
fn the_new_digest_shows_the_new_code_and_re_arms() {
    let mut w = ready_remembered(pair(), context());
    w.input(UserInput::Select(fp(2)));
    w.wait(100).digest(fp(2), "1111 2222 3333 4444");
    assert_eq!(w.state(), ConfirmState::Ready);
    w.wait(400);
    assert_eq!(w.click(), vec![]);
    w.wait(300);
    assert_eq!(
        w.click(),
        vec![Intent::Sign {
            fingerprint: fp(2),
            via: 0,
            remember: false
        }]
    );
}

#[test]
fn the_stale_digest_of_the_previous_certificate_is_ignored() {
    let mut w = ready_remembered(pair(), context());
    w.input(UserInput::Select(fp(2)));
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    assert_eq!(w.state(), ConfirmState::Choosing);
}

#[test]
fn a_digest_pending_notice_keeps_the_card_preparing() {
    use websign_ui_model::confirm::port::UiCommand;
    let mut w = choosing();
    w.apply(UiCommand::DigestPending {
        key: KEY,
        fingerprint: fp(1),
    });
    assert_eq!(w.state(), ConfirmState::Choosing);
    assert!(matches!(w.view().code, CodeCard::Preparing { .. }));
}
