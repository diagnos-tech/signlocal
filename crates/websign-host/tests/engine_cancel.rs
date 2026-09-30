//! SPEC §8.9 and §4 (`end`): cancel from the page, from the person, and by
//! disconnect; and what happens to a request after it ended.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN, wire};
use websign_core::HashAlgorithm;
use websign_host::{Control, EngineEvent};
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::Finish;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

#[test]
fn cancel_from_the_client_aborts_the_request() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);

    h.send(wire::cancel("s1"));
    let out = h.take();
    assert_eq!(out.only_error(), ("s1".to_owned(), ErrorCode::Aborted));
    assert_eq!(out.finished(), [(key, Finish::Aborted)]);
    assert!(out.signs().is_empty());
}

#[test]
fn cancel_works_in_every_early_state() {
    let p = Cert::p256();
    // Listing: before the key store answered.
    let mut h = Harness::native_ready();
    h.begin("a", ORIGIN, SHA256);
    h.send(wire::cancel("a"));
    assert_eq!(h.take().only_error().1, ErrorCode::Aborted);

    // Selecting (new site).
    let mut h = Harness::native_ready();
    h.begin("b", ORIGIN, SHA256);
    h.listed(&[&p]);
    h.send(wire::cancel("b"));
    let out = h.take();
    assert_eq!(out.only_error().1, ErrorCode::Aborted);
    assert!(out.need_digests().is_empty());

    // Awaiting the digest.
    let mut h = Harness::native_ready();
    h.remember(ORIGIN, &[&p]);
    h.begin("c", ORIGIN, SHA256);
    h.listed(&[&p]);
    h.send(wire::cancel("c"));
    assert_eq!(h.take().only_error().1, ErrorCode::Aborted);
}

#[test]
fn cancel_for_an_unknown_or_finished_id_is_accepted_and_ignored() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    assert_eq!(h.send(wire::cancel("never")), Control::Continue);
    assert!(h.take().is_silent());

    let key = h.drive_to_ready("s1", ORIGIN, &p);
    h.ui(UiEvent::Cancel {
        key,
        code: ErrorCode::UserCancelled,
    });
    h.take();
    assert_eq!(h.send(wire::cancel("s1")), Control::Continue);
    assert!(h.take().is_silent(), "the request already ended");
}

#[test]
fn cancel_by_the_person_sends_the_windows_code() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let out = h.ui_out(UiEvent::Cancel {
        key,
        code: ErrorCode::UserCancelled,
    });
    assert_eq!(
        out.only_error(),
        ("s1".to_owned(), ErrorCode::UserCancelled)
    );
    assert_eq!(out.finished().len(), 1);
}

#[test]
fn nothing_can_be_sent_for_a_request_after_it_ended() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    h.ui(UiEvent::Cancel {
        key,
        code: ErrorCode::UserCancelled,
    });
    h.take();

    let out = h.answer_digest("s1", 1, SHA256);
    assert_eq!(
        out.only_error().1,
        ErrorCode::InvalidRequest,
        "the id is no longer open"
    );
    assert!(h.press_sign(key, &p, None).signs().is_empty());
}

#[test]
fn a_disconnect_tells_the_window_and_sends_no_frames() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);

    let control = h.engine.handle(EngineEvent::Closed);
    let out = h.take();
    assert!(out.frames.is_empty(), "there is nobody to send to");
    assert_eq!(out.finished(), [(key, Finish::SiteCancelled)]);
    // One connection per process: a clean close ends it with status 0.
    assert_eq!(control, Control::Exit(0));
}

#[test]
fn a_broken_stream_ends_open_requests_like_a_disconnect() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    let control = h
        .engine
        .handle(EngineEvent::Broken("framing violated".to_owned()));
    let out = h.take();
    assert!(out.frames.is_empty());
    assert_eq!(out.finished(), [(key, Finish::SiteCancelled)]);
    assert_eq!(control, Control::Exit(1));
}

#[test]
fn a_disconnect_with_a_queue_ends_every_request() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.begin("a", ORIGIN, SHA256);
    h.begin("b", ORIGIN, SHA256);
    h.listed(&[&p]);
    h.engine.handle(EngineEvent::Closed);
    let out = h.take();
    assert!(out.frames.is_empty());
    assert_eq!(
        out.finished().len(),
        1,
        "only the visible request was ever on screen"
    );
    assert!(out.signs().is_empty());
}

#[test]
fn a_write_failure_does_not_stop_the_engine_from_cleaning_up() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, &p);
    h.rec.borrow_mut().client_gone = true;
    h.ui(UiEvent::Cancel {
        key,
        code: ErrorCode::UserCancelled,
    });
    let out = h.take();
    assert!(out.frames.is_empty(), "the fake refused the write");
    assert_eq!(
        out.finished().len(),
        1,
        "the window still closes the request"
    );
}

#[test]
fn a_closed_client_with_no_requests_just_ends() {
    let mut h = Harness::native_ready();
    let control = h.engine.handle(EngineEvent::Closed);
    assert_eq!(control, Control::Exit(0));
    assert!(h.take().frames.is_empty());
}
