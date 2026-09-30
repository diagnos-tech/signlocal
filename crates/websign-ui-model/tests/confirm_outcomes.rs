//! SPEC §2.2 and ux §4.8, §4.11: how a request ends (success hold, site
//! gave up, timeout, abort).

mod common;

use common::*;
use websign_ui_model::confirm::port::{Finish, RequestKey, UiCommand};
use websign_ui_model::confirm::{ConfirmState, UserInput};

fn signing() -> Window {
    let mut w = ready_remembered(pair(), context());
    assert_eq!(w.click().len(), 1);
    w.signing();
    assert_eq!(w.state(), ConfirmState::Signing);
    w
}

#[test]
fn a_signature_shows_success_for_900_ms_then_the_window_is_idle() {
    let mut w = signing();
    w.finish(Finish::Signed);
    assert_eq!(w.state(), ConfirmState::Success);
    w.wait(899);
    assert_eq!(w.tick(), vec![]);
    assert_eq!(w.state(), ConfirmState::Success);
    w.wait(1);
    w.tick();
    assert_eq!(w.state(), ConfirmState::Idle);
}

#[test]
fn ticking_early_never_ends_the_success_hold() {
    let mut w = signing();
    w.finish(Finish::Signed);
    for _ in 0..8 {
        w.wait(100);
        w.tick();
        assert_eq!(w.state(), ConfirmState::Success);
    }
}

#[test]
fn the_next_request_takes_over_during_the_success_hold() {
    let mut w = signing();
    w.finish(Finish::Signed);
    w.wait(300);
    let next = RequestKey(2);
    w.key = next;
    w.apply(UiCommand::Open(request(next, sign_mode(), false)));
    assert_eq!(w.state(), ConfirmState::LoadingCerts);
    w.wait(2_000);
    w.tick();
    assert_eq!(
        w.state(),
        ConfirmState::LoadingCerts,
        "the old hold is gone"
    );
}

#[test]
fn success_is_final_for_input() {
    let mut w = signing();
    w.finish(Finish::Signed);
    w.wait(100);
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.state(), ConfirmState::Success);
}

#[test]
fn a_site_that_gives_up_shows_the_notice_for_1500_ms() {
    let mut w = ready_remembered(pair(), context());
    w.finish(Finish::SiteCancelled);
    assert_eq!(w.state(), ConfirmState::SiteCancelled);
    w.wait(1_499);
    w.tick();
    assert_eq!(w.state(), ConfirmState::SiteCancelled);
    w.wait(1);
    w.tick();
    assert_eq!(w.state(), ConfirmState::Idle);
}

#[test]
fn a_site_can_give_up_in_any_state() {
    // Loading, choosing, ready, signing.
    let mut loading = Window::new();
    loading.open(request(KEY, sign_mode(), false));
    loading.finish(Finish::SiteCancelled);
    assert_eq!(loading.state(), ConfirmState::SiteCancelled);

    let mut choosing = Window::new();
    choosing.open(request(KEY, sign_mode(), false));
    choosing.wait(10).certificates(pair(), context());
    choosing.finish(Finish::SiteCancelled);
    assert_eq!(choosing.state(), ConfirmState::SiteCancelled);

    let mut in_signing = signing();
    in_signing.finish(Finish::SiteCancelled);
    assert_eq!(in_signing.state(), ConfirmState::SiteCancelled);
}

#[test]
fn a_timeout_shows_its_notice_for_1500_ms_then_closes() {
    let mut w = ready_remembered(pair(), context());
    w.finish(Finish::Timeout);
    assert_eq!(w.state(), ConfirmState::Timeout);
    assert_eq!(w.model.next_deadline(w.now), Some(w.now + ms(1_500)));
    w.wait(1_499);
    w.tick();
    assert_eq!(w.state(), ConfirmState::Timeout);
    w.wait(1);
    w.tick();
    assert_eq!(w.state(), ConfirmState::Idle);
}

#[test]
fn an_aborted_request_leaves_without_a_notice() {
    let mut w = ready_remembered(pair(), context());
    w.finish(Finish::Aborted);
    assert_eq!(w.state(), ConfirmState::Idle);
}

#[test]
fn hide_empties_the_window_from_any_state() {
    let mut w = signing();
    w.apply(UiCommand::Hide);
    assert_eq!(w.state(), ConfirmState::Idle);
    assert_eq!(w.input(UserInput::Escape), vec![]);
}

#[test]
fn escape_on_a_finish_notice_closes_it_without_a_cancel() {
    // The request is already answered: there is nothing left to cancel.
    let mut w = ready_remembered(pair(), context());
    w.finish(Finish::SiteCancelled);
    assert_eq!(w.input(UserInput::Escape), vec![]);
    assert_eq!(w.state(), ConfirmState::Idle);
}
