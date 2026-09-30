//! SPEC §6: the request queue as pure bookkeeping.

use websign_host::queue::{Busy, RequestQueue};
use websign_ui_model::confirm::port::RequestKey;

fn key(n: u64) -> RequestKey {
    RequestKey(n)
}

fn filled(waiting: u64) -> RequestQueue {
    let mut queue = RequestQueue::default();
    for n in 0..=waiting {
        queue.push(key(n)).expect("room");
    }
    queue
}

#[test]
fn an_idle_queue_reports_no_position() {
    let queue = RequestQueue::default();
    assert_eq!(queue.active(), None);
    assert_eq!(queue.position(), (0, 0));
}

#[test]
fn the_first_push_becomes_active_and_the_next_ones_wait() {
    let mut queue = RequestQueue::default();
    assert_eq!(queue.push(key(1)), Ok(true));
    assert_eq!(queue.push(key(2)), Ok(false));
    assert_eq!(queue.push(key(3)), Ok(false));
    assert_eq!(queue.active(), Some(key(1)));
    assert_eq!(queue.position(), (1, 3));
}

#[test]
fn ten_wait_and_the_eleventh_is_busy() {
    let mut queue = filled(10);
    assert_eq!(queue.position(), (1, 11));
    assert_eq!(queue.push(key(99)), Err(Busy));
    assert_eq!(queue.position(), (1, 11), "a refused push changes nothing");
}

#[test]
fn removing_the_active_promotes_the_oldest_waiting() {
    let mut queue = filled(3);
    assert_eq!(queue.remove(key(0)), Some(key(1)));
    assert_eq!(queue.active(), Some(key(1)));
    assert_eq!(queue.position(), (1, 3));
    assert_eq!(queue.remove(key(1)), Some(key(2)));
    assert_eq!(queue.remove(key(2)), Some(key(3)));
    assert_eq!(queue.position(), (1, 1));
}

#[test]
fn removing_the_last_request_leaves_the_queue_idle() {
    let mut queue = filled(0);
    assert_eq!(queue.remove(key(0)), None);
    assert_eq!(queue.active(), None);
    assert_eq!(queue.position(), (0, 0));
    assert_eq!(queue.push(key(5)), Ok(true), "and it can be used again");
}

#[test]
fn removing_a_waiting_request_keeps_the_order_of_the_others() {
    let mut queue = filled(3);
    assert_eq!(queue.remove(key(2)), None, "the active one did not change");
    assert_eq!(queue.active(), Some(key(0)));
    assert_eq!(queue.position(), (1, 3));
    assert_eq!(queue.remove(key(0)), Some(key(1)));
    assert_eq!(queue.remove(key(1)), Some(key(3)));
}

#[test]
fn removing_an_unknown_key_changes_nothing() {
    let mut queue = filled(2);
    assert_eq!(queue.remove(key(42)), None);
    assert_eq!(queue.active(), Some(key(0)));
    assert_eq!(queue.position(), (1, 3));
}

#[test]
fn a_freed_place_can_be_filled_again_after_a_full_queue() {
    let mut queue = filled(10);
    assert_eq!(queue.push(key(50)), Err(Busy));
    queue.remove(key(4));
    assert_eq!(queue.push(key(50)), Ok(false));
    assert_eq!(queue.push(key(51)), Err(Busy));
}

#[test]
fn busy_is_a_readable_error() {
    assert_eq!(Busy.to_string(), "too many requests are waiting");
}
