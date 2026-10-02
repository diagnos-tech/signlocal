//! SPEC §2.2 and §2.4: header, queue and the footer countdown.

mod common;

use common::*;
use websign_ui_model::confirm::ConfirmState;
use websign_ui_model::confirm::port::{Finish, RequestKey, UiCommand};
use websign_ui_model::confirm::view::FooterHint;

fn open_with_timeout(seconds: u32) -> Window {
    let mut req = request(KEY, sign_mode(), true);
    req.timeout_secs = seconds;
    let mut w = Window::new();
    w.open(req);
    w.wait(100).certificates(pair(), context());
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    w
}

#[test]
fn the_header_carries_the_caller_the_mode_and_the_queue() {
    let mut req = request(KEY, sign_mode(), false);
    req.position = (2, 3);
    let mut w = Window::new();
    w.open(req);
    let view = w.view();
    assert_eq!(view.header.caller, caller());
    assert!(!view.header.remembered);
    assert_eq!(view.header.queue, Some((2, 3)));
    assert_eq!(view.mode, sign_mode());
}

#[test]
fn a_single_request_has_no_queue_line() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    assert_eq!(w.view().header.queue, None);
}

#[test]
fn a_queue_update_changes_the_line_and_never_the_state() {
    let mut w = ready_remembered(pair(), context());
    w.apply(UiCommand::Queue {
        key: KEY,
        position: (1, 4),
    });
    assert_eq!(w.view().header.queue, Some((1, 4)));
    assert_eq!(w.state(), ConfirmState::Ready);
    w.apply(UiCommand::Queue {
        key: KEY,
        position: (1, 1),
    });
    assert_eq!(w.view().header.queue, None);
}

#[test]
fn a_queue_update_for_another_request_is_ignored() {
    let mut w = ready_remembered(pair(), context());
    w.apply(UiCommand::Queue {
        key: RequestKey(99),
        position: (1, 5),
    });
    assert_eq!(w.view().header.queue, None);
}

#[test]
fn the_primary_button_comes_first_only_on_windows() {
    let w = ready_remembered(pair(), context());
    assert_eq!(w.view().footer.primary_first, cfg!(target_os = "windows"));
}

#[test]
fn the_countdown_starts_when_30_seconds_remain() {
    let mut w = open_with_timeout(40);
    w.at(9_000);
    assert!(!matches!(
        w.view().footer.hint,
        FooterHint::ExpiresIn { .. }
    ));
    w.at(10_000);
    assert_eq!(w.view().footer.hint, FooterHint::ExpiresIn { seconds: 30 });
    w.at(25_000);
    assert_eq!(w.view().footer.hint, FooterHint::ExpiresIn { seconds: 15 });
    w.at(39_000);
    assert_eq!(w.view().footer.hint, FooterHint::ExpiresIn { seconds: 1 });
}

#[test]
fn a_request_shorter_than_30_seconds_counts_down_from_the_start() {
    let mut w = open_with_timeout(20);
    w.at(5_000);
    assert_eq!(w.view().footer.hint, FooterHint::ExpiresIn { seconds: 15 });
}

#[test]
fn a_queue_update_does_not_restart_the_timeout() {
    let mut w = open_with_timeout(40);
    w.at(20_000);
    w.apply(UiCommand::Queue {
        key: KEY,
        position: (1, 2),
    });
    assert_eq!(w.view().footer.hint, FooterHint::ExpiresIn { seconds: 20 });
}

#[test]
fn a_new_open_restarts_the_timeout() {
    let mut w = open_with_timeout(40);
    w.at(20_000);
    w.click();
    w.signing().finish(Finish::Signed);
    let mut next = request(RequestKey(2), sign_mode(), true);
    next.timeout_secs = 40;
    w.open(next);
    assert!(!matches!(
        w.view().footer.hint,
        FooterHint::ExpiresIn { .. }
    ));
}

#[test]
fn a_partial_second_rounds_up() {
    let mut w = open_with_timeout(40);
    w.at(25_500);
    assert_eq!(w.view().footer.hint, FooterHint::ExpiresIn { seconds: 15 });
}
