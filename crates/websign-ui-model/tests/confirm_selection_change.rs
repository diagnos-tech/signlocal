//! SPEC §2.2 with D11: changing the certificate of a caller that is not
//! remembered asks for Continue again, and stale digests are ignored.

mod common;

use common::*;
use websign_ui_model::confirm::view::{CodeCard, PrimaryButton};
use websign_ui_model::confirm::{ConfirmState, Intent, UserInput};

#[test]
fn changing_the_certificate_goes_back_to_continue() {
    let mut w = ready_new_caller(pair(), context());
    assert_eq!(
        w.input(UserInput::Select(fp(2))),
        vec![Intent::Selected(fp(2))]
    );
    assert_eq!(w.state(), ConfirmState::Choosing);
    let view = w.view();
    assert_eq!(view.selected, Some(fp(2)));
    assert_eq!(view.code, CodeCard::ContinueHint);
    assert_eq!(view.footer.primary, PrimaryButton::Continue);
}

#[test]
fn the_new_certificate_needs_its_own_continue_and_digest() {
    let mut w = ready_new_caller(pair(), context());
    w.input(UserInput::Select(fp(2)));
    w.wait(1000);
    assert_eq!(w.click(), vec![Intent::Continue(fp(2))]);

    // The digest of the old certificate no longer matches.
    w.wait(50).digest(fp(1), "7F3A 9C21 E0B4 55D8");
    assert_eq!(w.state(), ConfirmState::Choosing);
    w.digest(fp(2), "1111 2222 3333 4444");
    assert_eq!(w.state(), ConfirmState::Ready);
}

#[test]
fn a_digest_for_the_previous_selection_after_a_change_is_ignored() {
    let mut w = ready_new_caller(pair(), context());
    w.input(UserInput::Select(fp(2)));
    w.wait(10).digest(fp(1), "7F3A 9C21 E0B4 55D8");
    assert_eq!(w.state(), ConfirmState::Choosing);
}

#[test]
fn selection_is_ignored_while_unarmed() {
    // Only leaving works while unarmed; selecting is a click like any other.
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(100).certificates(pair(), context());
    w.wait(100);
    assert_eq!(w.input(UserInput::Select(fp(2))), vec![]);
    assert_eq!(w.view().selected, Some(fp(1)));
}
