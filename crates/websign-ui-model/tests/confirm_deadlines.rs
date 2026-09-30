//! SPEC §2.2: `next_deadline`, the instant the renderer schedules its next
//! repaint for: the earliest of arming completion, a result hold's end, the
//! countdown's next second and the 150 ms skeleton delay.

mod common;

use common::*;
use websign_ui_model::confirm::port::Finish;
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
    // Opened at 0 with 60 s: at 40.2 s the footer shows 20 (19.8 s left,
    // rounded up) and drops to 19 at 41 s.
    let mut w = open_with_timeout(60);
    w.at(40_200);
    assert_eq!(w.view().footer.hint, FooterHint::ExpiresIn { seconds: 20 });
    assert_eq!(w.model.next_deadline(w.now), Some(w.start + ms(41_000)));
}

#[test]
fn the_skeleton_delay_is_a_deadline() {
    // Remembered caller: preparing since the listing at 100 ms; arming (focus
    // at 0) ends at 600 ms, the skeleton appears at 250 ms.
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), true));
    w.wait(100).certificates(pair(), context());
    w.wait(20);
    assert_eq!(w.model.next_deadline(w.now), Some(w.start + ms(250)));
    w.at(300);
    assert_eq!(w.model.next_deadline(w.now), Some(w.start + ms(600)));
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
    // 1.1 s in; the request times out after 300 s and the countdown starts
    // at 270 s.
    assert_eq!(w.model.next_deadline(w.now), Some(w.start + ms(270_000)));
}
