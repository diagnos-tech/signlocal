//! SPEC §2.2 with D11: a caller that is not remembered gets the certificate
//! only after the person presses "Continue".

mod common;

use common::*;
use websign_protocol::types::HashName;
use websign_ui_model::confirm::view::{CodeCard, PrimaryButton};
use websign_ui_model::confirm::{ConfirmState, Intent, UserInput};

/// New caller, list loaded (Ana selected), long past arming.
fn choosing() -> Window {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(100).certificates(pair(), context());
    w.wait(1000);
    w
}

#[test]
fn opening_shows_the_loading_state() {
    let mut w = Window::new();
    assert_eq!(w.state(), ConfirmState::Idle);
    w.open(request(KEY, sign_mode(), false));
    assert_eq!(w.state(), ConfirmState::LoadingCerts);
}

#[test]
fn a_listing_with_usable_rows_leads_to_choosing_with_the_initial_selection() {
    let w = choosing();
    assert_eq!(w.state(), ConfirmState::Choosing);
    let view = w.view();
    assert_eq!(view.selected, Some(fp(1)));
    assert_eq!(view.list.as_ref().unwrap().usable.len(), 2);
}

#[test]
fn before_continue_the_card_hints_and_the_button_says_continue() {
    let view = choosing().view();
    assert_eq!(view.code, CodeCard::ContinueHint);
    assert_eq!(view.footer.primary, PrimaryButton::Continue);
    assert!(view.footer.primary_enabled);
}

#[test]
fn the_button_is_disabled_until_armed() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(100).certificates(pair(), context());
    w.wait(200);
    assert!(!w.view().footer.primary_enabled);
    w.wait(1000);
    assert!(w.view().footer.primary_enabled);
}

#[test]
fn a_click_on_continue_releases_the_selected_certificate() {
    let mut w = choosing();
    assert_eq!(w.click(), vec![Intent::Continue(fp(1))]);
    assert_eq!(w.state(), ConfirmState::Choosing);
    assert_eq!(w.view().code, CodeCard::Preparing { skeleton: false });
}

#[test]
fn a_click_before_arming_is_discarded() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(100).certificates(pair(), context());
    w.wait(200);
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.view().code, CodeCard::ContinueHint);
}

#[test]
fn enter_on_the_list_never_releases_or_signs() {
    let mut w = choosing();
    assert_eq!(w.input(UserInput::Enter), vec![]);
    assert_eq!(w.state(), ConfirmState::Choosing);
}

#[test]
fn the_digest_of_the_selected_certificate_shows_the_code() {
    let mut w = choosing();
    w.click();
    w.wait(50).digest(fp(1), "7F3A 9C21 E0B4 55D8");
    assert_eq!(w.state(), ConfirmState::Ready);
    let view = w.view();
    assert_eq!(
        view.code,
        CodeCard::Ready {
            code: code("7F3A 9C21 E0B4 55D8"),
            hash: HashName::Sha256,
        }
    );
    assert_eq!(view.footer.primary, PrimaryButton::Sign);
}

#[test]
fn the_ready_code_uses_the_requested_hash() {
    let mut w = Window::new();
    let mode = websign_ui_model::confirm::port::Mode::Sign {
        hash: HashName::Sha384,
    };
    w.open(request(KEY, mode, false));
    w.wait(100).certificates(pair(), context());
    w.wait(1000);
    w.click();
    w.digest(fp(1), "AAAA BBBB CCCC DDDD");
    let CodeCard::Ready { hash, .. } = w.view().code else {
        panic!("code is not ready");
    };
    assert_eq!(hash, HashName::Sha384);
}

#[test]
fn a_digest_for_a_certificate_that_is_not_selected_is_ignored() {
    let mut w = choosing();
    w.click();
    w.wait(50).digest(fp(2), "0000 1111 2222 3333");
    assert_eq!(w.state(), ConfirmState::Choosing);
    assert_eq!(w.view().code, CodeCard::Preparing { skeleton: false });
}

#[test]
fn signing_needs_a_fresh_arming_after_the_digest() {
    let mut w = choosing();
    w.click();
    w.wait(50).digest(fp(1), "7F3A 9C21 E0B4 55D8");
    w.wait(300);
    assert!(!w.view().footer.primary_enabled);
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.state(), ConfirmState::Ready);
    w.wait(300);
    assert!(w.view().footer.primary_enabled);
    assert_eq!(
        w.click(),
        vec![Intent::Sign {
            fingerprint: fp(1),
            via: 0,
            remember: false
        }]
    );
    assert_eq!(w.state(), ConfirmState::Signing);
    assert_eq!(w.view().footer.primary, PrimaryButton::Signing);
    assert!(!w.view().footer.primary_enabled);
}

#[test]
fn enter_signs_a_ready_request_without_our_pin_field() {
    let mut w = ready_new_caller(pair(), context());
    assert_eq!(
        w.input(UserInput::Enter),
        vec![Intent::Sign {
            fingerprint: fp(1),
            via: 0,
            remember: false
        }]
    );
    assert_eq!(w.state(), ConfirmState::Signing);
}

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
    // SPEC §2.2: "inputs other than Escape are ignored while unarmed".
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(100).certificates(pair(), context());
    w.wait(100);
    assert_eq!(w.input(UserInput::Select(fp(2))), vec![]);
    assert_eq!(w.view().selected, Some(fp(1)));
}

#[test]
fn a_list_with_a_last_used_certificate_selects_it() {
    let mut ctx = context();
    ctx.last_used_here = Some(fp(2));
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(100).certificates(pair(), ctx);
    w.wait(1000);
    assert_eq!(w.view().selected, Some(fp(2)));
    assert_eq!(w.click(), vec![Intent::Continue(fp(2))]);
}
