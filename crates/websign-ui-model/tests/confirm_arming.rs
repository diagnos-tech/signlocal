//! SPEC §2.1 and ux §4.7: the 600 ms arming of the primary button.

mod common;

use std::time::Instant;

use common::ms;
use websign_ui_model::confirm::arming::{ARMING_DELAY, Arming};

fn armed_at(t0: Instant) -> Arming {
    let mut arming = Arming::default();
    arming.rearm(t0);
    arming
}

#[test]
fn the_delay_is_600_milliseconds() {
    assert_eq!(ARMING_DELAY, ms(600));
}

#[test]
fn a_fresh_arming_is_not_armed_and_has_no_deadline() {
    let arming = Arming::default();
    assert!(!arming.is_armed(Instant::now()));
    assert_eq!(arming.armed_at(), None);
}

#[test]
fn it_arms_exactly_600_ms_after_the_rearm() {
    let t0 = Instant::now();
    let arming = armed_at(t0);
    assert!(!arming.is_armed(t0));
    assert!(!arming.is_armed(t0 + ms(599)));
    assert!(arming.is_armed(t0 + ms(600)));
    assert!(arming.is_armed(t0 + ms(60_000)));
    assert_eq!(arming.armed_at(), Some(t0 + ms(600)));
}

#[test]
fn rearming_restarts_the_delay() {
    let t0 = Instant::now();
    let mut arming = armed_at(t0);
    arming.rearm(t0 + ms(1_000));
    assert!(!arming.is_armed(t0 + ms(1_599)));
    assert!(arming.is_armed(t0 + ms(1_600)));
    assert_eq!(arming.armed_at(), Some(t0 + ms(1_600)));
}

#[test]
fn disarming_stops_the_count_until_the_next_rearm() {
    let t0 = Instant::now();
    let mut arming = armed_at(t0);
    arming.disarm();
    assert!(!arming.is_armed(t0 + ms(10_000)));
    assert_eq!(arming.armed_at(), None);
    arming.rearm(t0 + ms(10_000));
    assert!(!arming.is_armed(t0 + ms(10_500)));
    assert!(arming.is_armed(t0 + ms(10_600)));
}

#[test]
fn a_click_needs_an_armed_press_and_an_armed_release() {
    let t0 = Instant::now();
    let mut arming = armed_at(t0);
    arming.press(t0 + ms(700));
    assert!(arming.release(t0 + ms(800)));
}

#[test]
fn a_press_before_arming_never_counts_even_if_released_after() {
    let t0 = Instant::now();
    let mut arming = armed_at(t0);
    arming.press(t0 + ms(300));
    assert!(!arming.release(t0 + ms(900)));
}

#[test]
fn a_release_without_a_press_does_not_count() {
    let t0 = Instant::now();
    let mut arming = armed_at(t0);
    assert!(!arming.release(t0 + ms(900)));
}

#[test]
fn a_release_clears_the_press_whatever_it_returns() {
    let t0 = Instant::now();
    let mut arming = armed_at(t0);
    arming.press(t0 + ms(700));
    assert!(arming.release(t0 + ms(710)));
    assert!(!arming.release(t0 + ms(720)), "one press, one click");

    arming.press(t0 + ms(100));
    assert!(!arming.release(t0 + ms(200)));
    assert!(
        !arming.release(t0 + ms(900)),
        "the unarmed press is gone too"
    );
}

#[test]
fn a_rearm_between_press_and_release_cancels_the_click() {
    let t0 = Instant::now();
    let mut arming = armed_at(t0);
    arming.press(t0 + ms(700));
    arming.rearm(t0 + ms(800));
    assert!(!arming.release(t0 + ms(2_000)));
}

#[test]
fn a_release_while_disarmed_does_not_count() {
    let t0 = Instant::now();
    let mut arming = armed_at(t0);
    arming.press(t0 + ms(700));
    arming.disarm();
    assert!(!arming.release(t0 + ms(2_000)));
}

#[test]
fn a_press_while_disarmed_is_not_armed() {
    let t0 = Instant::now();
    let mut arming = armed_at(t0);
    arming.disarm();
    arming.press(t0 + ms(5_000));
    assert!(!arming.release(t0 + ms(5_100)));
}
