//! SPEC §2.2 with D11: a caller that is not remembered gets the certificate
//! only after the person presses "Continue".

mod common;

use common::*;
use websign_protocol::types::HashName;
use websign_ui_model::confirm::port::UiCommand;
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
fn a_digest_before_continue_is_ignored() {
    // The window never asked for it: the certificate was not released yet.
    let mut w = choosing();
    w.apply(UiCommand::DigestPending {
        key: KEY,
        fingerprint: fp(1),
    });
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    assert_eq!(w.state(), ConfirmState::Choosing);
    assert_eq!(w.view().code, CodeCard::ContinueHint);
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
