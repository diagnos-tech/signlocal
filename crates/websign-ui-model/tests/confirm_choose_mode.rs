//! ux §4.10 and SPEC §2.2: Choose mode (`certificates()` from a site that is
//! not remembered) and the "Remember this site" box.

mod common;

use common::*;
use websign_ui_model::confirm::port::{Finish, Mode};
use websign_ui_model::confirm::view::{CodeCard, PinBlock, PrimaryButton, RememberBox};
use websign_ui_model::confirm::{ConfirmState, Intent, UserInput};

fn chooser() -> Window {
    let mut w = Window::new();
    w.open(request(KEY, Mode::Choose, false));
    w.wait(100).certificates(pair(), context());
    w.wait(1_000);
    w
}

#[test]
fn choose_mode_shows_what_the_site_receives_instead_of_a_code() {
    let w = chooser();
    let view = w.view();
    assert_eq!(w.state(), ConfirmState::Choosing);
    assert_eq!(view.mode, Mode::Choose);
    assert_eq!(view.code, CodeCard::SelectShares);
    assert_eq!(view.pin, PinBlock::Hidden);
    assert_eq!(view.footer.primary, PrimaryButton::UseCertificate);
    assert!(view.footer.primary_enabled);
}

#[test]
fn a_click_sends_the_chosen_certificate() {
    let mut w = chooser();
    assert_eq!(
        w.click(),
        vec![Intent::Choose {
            fingerprint: fp(1),
            remember: false
        }]
    );
}

#[test]
fn the_selected_certificate_is_the_one_sent() {
    let mut w = chooser();
    w.input(UserInput::Select(fp(2)));
    w.wait(1_000);
    assert_eq!(w.view().code, CodeCard::SelectShares);
    assert_eq!(
        w.click(),
        vec![Intent::Choose {
            fingerprint: fp(2),
            remember: false
        }]
    );
}

#[test]
fn choosing_needs_the_same_600_ms_arming() {
    let mut w = Window::new();
    w.open(request(KEY, Mode::Choose, false));
    w.wait(100).certificates(pair(), context());
    w.wait(200);
    assert!(!w.view().footer.primary_enabled);
    assert_eq!(w.click(), vec![]);
    w.wait(1_000);
    assert_eq!(w.click().len(), 1);
}

#[test]
fn enter_on_the_list_does_not_choose() {
    let mut w = chooser();
    assert_eq!(w.input(UserInput::Enter), vec![]);
}

#[test]
fn the_remember_choice_travels_with_the_decision() {
    let mut w = chooser();
    w.input(UserInput::Remember(true));
    assert_eq!(
        w.click(),
        vec![Intent::Choose {
            fingerprint: fp(1),
            remember: true
        }]
    );
}

#[test]
fn the_chosen_certificate_ends_in_success_and_closes_after_900_ms() {
    let mut w = chooser();
    w.click();
    w.finish(Finish::Chosen);
    assert_eq!(w.state(), ConfirmState::Success);
    w.wait(900);
    w.tick();
    assert_eq!(w.state(), ConfirmState::Idle);
}

#[test]
fn a_new_site_sees_the_remember_box_unchecked() {
    let view = chooser().view();
    assert_eq!(view.remember, RememberBox::Enabled { checked: false });
}

#[test]
fn ticking_the_box_is_reflected_in_the_view_and_can_be_undone() {
    let mut w = chooser();
    w.input(UserInput::Remember(true));
    assert_eq!(w.view().remember, RememberBox::Enabled { checked: true });
    w.input(UserInput::Remember(false));
    assert_eq!(w.view().remember, RememberBox::Enabled { checked: false });
}

#[test]
fn a_site_that_cannot_be_remembered_gets_a_disabled_box() {
    let mut req = request(KEY, Mode::Choose, false);
    req.can_remember = false;
    let mut w = Window::new();
    w.open(req);
    w.wait(100).certificates(pair(), context());
    w.wait(1_000);
    assert_eq!(w.view().remember, RememberBox::Disabled);
    w.input(UserInput::Remember(true));
    assert_eq!(w.view().remember, RememberBox::Disabled);
    assert_eq!(
        w.click(),
        vec![Intent::Choose {
            fingerprint: fp(1),
            remember: false
        }]
    );
}

#[test]
fn a_remembered_caller_never_sees_the_box() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), true));
    w.wait(100).certificates(pair(), context());
    assert_eq!(w.view().remember, RememberBox::Hidden);
}

#[test]
fn the_box_is_offered_in_sign_mode_for_a_new_caller_and_travels_with_sign() {
    let mut w = ready_new_caller(pair(), context());
    assert_eq!(w.view().remember, RememberBox::Enabled { checked: false });
    w.input(UserInput::Remember(true));
    assert_eq!(
        w.input(UserInput::Enter),
        vec![Intent::Sign {
            fingerprint: fp(1),
            via: 0,
            remember: true
        }]
    );
}

#[test]
fn the_remember_choice_survives_a_change_of_certificate() {
    let mut w = ready_new_caller(pair(), context());
    w.input(UserInput::Remember(true));
    w.input(UserInput::Select(fp(2)));
    assert_eq!(w.view().remember, RememberBox::Enabled { checked: true });
}
