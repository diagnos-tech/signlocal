//! SPEC §8.2 and §2.2: who may say what on each transport.

mod common;

use common::harness::{Harness, hash_str};
use common::{ORIGIN, wire};
use websign_core::HashAlgorithm;
use websign_protocol::ErrorCode;

fn assert_refused(h: &mut Harness, frame: Vec<u8>, id: &str, code: ErrorCode) {
    h.send(frame);
    let out = h.take();
    assert_eq!(out.only_error(), (id.to_owned(), code));
    assert!(out.ui.is_empty(), "a refused request opens no window");
    assert!(
        out.keys.is_empty(),
        "a refused request never reaches the key store"
    );
}

#[test]
fn native_sign_begin_without_web_is_invalid() {
    let mut h = Harness::native_ready();
    let frame = wire::sign_begin("s", None, hash_str(HashAlgorithm::Sha256));
    assert_refused(&mut h, frame, "s", ErrorCode::InvalidRequest);
}

#[test]
fn native_status_and_choose_without_web_are_invalid() {
    let mut h = Harness::native_ready();
    assert_refused(
        &mut h,
        wire::status("a", None),
        "a",
        ErrorCode::InvalidRequest,
    );
    assert_refused(
        &mut h,
        wire::choose("b", None),
        "b",
        ErrorCode::InvalidRequest,
    );
}

#[test]
fn desktop_requests_with_web_are_invalid() {
    let mut h = Harness::desktop_ready();
    let frame = wire::sign_begin("s", Some(ORIGIN), "SHA-256");
    assert_refused(&mut h, frame, "s", ErrorCode::InvalidRequest);
    assert_refused(
        &mut h,
        wire::status("a", Some(ORIGIN)),
        "a",
        ErrorCode::InvalidRequest,
    );
    assert_refused(
        &mut h,
        wire::choose("b", Some(ORIGIN)),
        "b",
        ErrorCode::InvalidRequest,
    );
}

#[test]
fn an_insecure_origin_is_refused_with_insecure_origin() {
    let mut h = Harness::native_ready();
    for (id, origin) in [
        ("a", "http://evil.example"),
        ("b", "http://192.0.2.10"),
        ("c", "chrome-extension://abcdefghijklmnopabcdefghijklmnop"),
    ] {
        let frame = wire::sign_begin(id, Some(origin), "SHA-256");
        assert_refused(&mut h, frame, id, ErrorCode::InsecureOrigin);
    }
}

#[test]
fn an_insecure_top_origin_is_refused_too() {
    let mut h = Harness::native_ready();
    let web = wire::framed(ORIGIN, "http://evil.example");
    assert_refused(
        &mut h,
        wire::sign_begin_raw_web("s", web, "SHA-256"),
        "s",
        ErrorCode::InsecureOrigin,
    );
}

#[test]
fn http_localhost_is_a_secure_context() {
    let mut h = Harness::native_ready();
    h.send(wire::sign_begin(
        "s",
        Some("http://localhost:3000"),
        "SHA-256",
    ));
    let out = h.take();
    assert!(out.errors().is_empty());
    assert_eq!(out.opens().len(), 1);
}

#[test]
fn a_malformed_origin_is_an_invalid_request() {
    let mut h = Harness::native_ready();
    let frame = wire::sign_begin("s", Some("not an origin"), "SHA-256");
    assert_refused(&mut h, frame, "s", ErrorCode::InvalidRequest);
}

#[test]
fn status_and_choose_check_the_origin_as_well() {
    let mut h = Harness::native_ready();
    let bad = "http://evil.example";
    assert_refused(
        &mut h,
        wire::status("a", Some(bad)),
        "a",
        ErrorCode::InsecureOrigin,
    );
    assert_refused(
        &mut h,
        wire::choose("b", Some(bad)),
        "b",
        ErrorCode::InsecureOrigin,
    );
}

#[test]
fn diagnostics_open_is_allowed_on_both_transports() {
    for mut h in [Harness::native_ready(), Harness::desktop_ready()] {
        h.send(wire::diagnostics("d", None));
        let out = h.take();
        assert_eq!(out.kinds(), ["done"]);
        assert_eq!(out.diagnostics, [None]);
    }
}
