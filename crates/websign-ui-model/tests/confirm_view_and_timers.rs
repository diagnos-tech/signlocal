//! SPEC §2.2 and §2.4: header, queue, footer countdown and the deadlines the
//! renderer schedules repaints with.

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
fn an_idle_window_has_no_deadline() {
    let w = Window::new();
    assert_eq!(w.model.next_deadline(w.now), None);
}

#[test]
fn the_deadline_while_unarmed_is_the_end_of_arming() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), true));
    w.wait(50).certificates(pair(), context());
    w.wait(50);
    w.digest(fp(1), "7F3A 9C21 E0B4 55D8");
    let digest_at = w.now;
    w.wait(100);
    assert_eq!(w.model.next_deadline(w.now), Some(digest_at + ms(600)));
}

#[test]
fn the_deadline_during_the_success_hold_is_its_end() {
    let mut w = ready_remembered(pair(), context());
    w.click();
    w.signing();
    w.finish(Finish::Signed);
    let finished = w.now;
    w.wait(200);
    assert_eq!(w.model.next_deadline(w.now), Some(finished + ms(900)));
}

#[test]
fn the_deadline_during_the_site_cancelled_hold_is_its_end() {
    let mut w = ready_remembered(pair(), context());
    w.finish(Finish::SiteCancelled);
    let finished = w.now;
    w.wait(200);
    assert_eq!(w.model.next_deadline(w.now), Some(finished + ms(1_500)));
}

#[test]
fn in_the_last_30_seconds_the_deadline_is_the_next_countdown_second() {
    let mut w = open_with_timeout(60);
    w.at(40_000);
    w.wait(200);
    let deadline = w.model.next_deadline(w.now).expect("countdown is running");
    assert!(deadline > w.now);
    assert!(deadline <= w.now + ms(1_000), "next second boundary");
}

#[test]
fn the_deadline_is_never_in_the_past() {
    let mut w = ready_remembered(pair(), context());
    for step in [0, 10, 300, 5_000, 200_000] {
        w.wait(step);
        if let Some(deadline) = w.model.next_deadline(w.now) {
            assert!(deadline >= w.now, "{deadline:?} < {:?}", w.now);
        }
    }
}

#[test]
fn a_fully_armed_ready_window_far_from_the_timeout_waits_for_nothing_soon() {
    let w = ready_remembered(pair(), context());
    // 1.2 s in; the request times out after 300 s and the countdown starts at 270 s.
    let deadline = w.model.next_deadline(w.now);
    assert!(deadline.is_none_or(|d| d >= w.start + ms(269_000)));
}
