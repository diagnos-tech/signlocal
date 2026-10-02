//! SPEC §8.1 and §1: `hello`, negotiation, and the first-frame rules.

mod common;

use common::harness::{Harness, allowed_launch, foreign_launch};
use common::{ORIGIN, wire};
use websign_host::Control;
use websign_host::launch::{BrowserFamily, BrowserLaunch};
use websign_host::session::Transport;
use websign_protocol::types::BrowserName;
use websign_protocol::{AppMessage, ErrorCode};

/// A refused first frame ends the process with status 1: nothing about the
/// peer can be trusted after it.
fn is_exit(control: Control) -> bool {
    control == Control::Exit(1)
}

#[test]
fn hello_is_answered_with_the_negotiated_protocol_and_app_info() {
    let mut h = Harness::native();
    let control = h.send(wire::hello_native("h1", 1, 1));
    let out = h.take();

    assert_eq!(control, Control::Continue);
    assert_eq!(out.frames.len(), 1);
    let frame = &out.frames[0];
    assert_eq!(frame.id.as_str(), "h1");
    assert_eq!(frame.v, 1);
    match &frame.message {
        AppMessage::Hello(reply) => {
            assert_eq!(reply.protocol, 1);
            assert_eq!(reply.app.version, "1.4.0");
        }
        other => panic!("expected hello, got {other:?}"),
    }
    assert!(
        out.ui.is_empty() && out.keys.is_empty(),
        "hello touches neither window nor keys"
    );
}

#[test]
fn hello_over_desktop_is_answered_too() {
    let mut h = Harness::desktop();
    h.send(wire::hello_desktop("h", 1, 1));
    assert_eq!(h.take().kinds(), ["hello"]);
}

#[test]
fn the_highest_common_version_is_chosen() {
    let mut h = Harness::new(
        Transport::NativeMessaging {
            launch: allowed_launch(),
        },
        (1, 3),
    );
    h.send(wire::hello_native("h", 1, 2));
    let out = h.take();
    match &out.frames[0].message {
        AppMessage::Hello(reply) => assert_eq!(reply.protocol, 2),
        other => panic!("expected hello, got {other:?}"),
    }
}

#[test]
fn a_client_that_only_speaks_newer_versions_gets_app_outdated_and_the_stream_closes() {
    let mut h = Harness::native();
    let control = h.send(wire::hello_native("h", 2, 3));
    let out = h.take();
    assert_eq!(out.only_error(), ("h".to_owned(), ErrorCode::AppOutdated));
    assert_eq!(
        out.frames[0].v, 2,
        "protocol SPEC §5.1: the refusal speaks the hello's version"
    );
    assert!(
        is_exit(control),
        "a failed negotiation closes the connection"
    );
}

#[test]
fn a_client_that_only_speaks_older_versions_gets_client_outdated() {
    let mut h = Harness::new(
        Transport::NativeMessaging {
            launch: allowed_launch(),
        },
        (2, 3),
    );
    let control = h.send(wire::hello_native("h", 1, 1));
    let out = h.take();
    assert_eq!(
        out.only_error(),
        ("h".to_owned(), ErrorCode::ClientOutdated)
    );
    assert!(is_exit(control));
}

#[test]
fn a_malformed_version_range_is_an_invalid_request() {
    let mut h = Harness::native();
    let control = h.send(wire::hello_native("h", 3, 2));
    assert_eq!(h.take().only_error().1, ErrorCode::InvalidRequest);
    assert!(is_exit(control));
}

#[test]
fn status_before_hello_is_an_error_and_closes() {
    let mut h = Harness::native();
    let control = h.send(wire::status("s", Some(ORIGIN)));
    let out = h.take();
    assert_eq!(
        out.only_error(),
        ("s".to_owned(), ErrorCode::InvalidRequest)
    );
    assert!(is_exit(control));
    assert!(out.ui.is_empty(), "nothing opens before hello");
}

#[test]
fn sign_begin_before_hello_opens_nothing() {
    let mut h = Harness::native();
    let control = h.send(wire::sign_begin("s", Some(ORIGIN), "SHA-256"));
    let out = h.take();
    assert_eq!(out.only_error().1, ErrorCode::InvalidRequest);
    assert!(out.ui.is_empty() && out.keys.is_empty());
    assert!(is_exit(control));
}

#[test]
fn a_frame_without_a_usable_id_cannot_be_answered_and_closes() {
    let mut h = Harness::native();
    let control = h.send(b"{\"v\":1,\"type\":\"hello\"}".to_vec());
    let out = h.take();
    assert!(out.frames.is_empty(), "no id, nobody to answer");
    assert!(is_exit(control));
}

#[test]
fn garbage_before_hello_closes_without_a_reply() {
    let mut h = Harness::native();
    let control = h.send(b"not json".to_vec());
    assert!(h.take().frames.is_empty());
    assert!(is_exit(control));
}

#[test]
fn a_second_hello_is_refused_but_the_connection_stays_usable() {
    let mut h = Harness::native_ready();
    let control = h.send(wire::hello_native("again", 1, 1));
    let out = h.take();
    assert_eq!(
        out.only_error(),
        ("again".to_owned(), ErrorCode::InvalidRequest)
    );
    assert_eq!(
        control,
        Control::Continue,
        "protocol.md §3: the connection stays open"
    );

    h.send(wire::status("s", Some(ORIGIN)));
    assert_eq!(h.take().kinds(), ["status"]);
}

#[test]
fn unknown_fields_are_rejected_and_the_error_reaches_the_request_id() {
    let mut h = Harness::native();
    h.send(wire::hello_with_extra_field("h"));
    let out = h.take();
    assert_eq!(
        out.only_error(),
        ("h".to_owned(), ErrorCode::InvalidRequest)
    );
}

#[test]
fn native_messaging_hello_needs_browser_and_desktop_hello_refuses_it() {
    let mut native = Harness::native();
    let control = native.send(wire::hello_without_browser("h"));
    assert_eq!(native.take().only_error().1, ErrorCode::InvalidRequest);
    assert!(is_exit(control), "a refused hello closes like a failed one");

    let mut desktop = Harness::desktop();
    let control = desktop.send(wire::hello_native("h", 1, 1));
    assert_eq!(desktop.take().only_error().1, ErrorCode::InvalidRequest);
    assert!(is_exit(control));
}

#[test]
fn a_native_hello_records_the_connection_for_diagnostics() {
    let mut h = Harness::native();
    h.send(wire::hello_native("h", 1, 1));
    let state = h.state.borrow();
    assert_eq!(state.connections.len(), 1);
    assert_eq!(state.connections[0].browser, BrowserName::Chrome);
    assert_eq!(state.connections[0].browser_version, "129.0");
    assert_eq!(state.connections[0].extension_version, "1.4.2");
}

#[test]
fn a_desktop_hello_records_no_browser_connection() {
    let mut h = Harness::desktop();
    h.send(wire::hello_desktop("h", 1, 1));
    assert!(h.state.borrow().connections.is_empty());
}

#[test]
fn a_launch_by_an_unknown_extension_is_refused_on_the_first_frame() {
    let mut h = Harness::new(
        Transport::NativeMessaging {
            launch: foreign_launch(),
        },
        (1, 1),
    );
    let control = h.send(wire::hello_native("h", 1, 1));
    let out = h.take();
    assert_eq!(
        out.only_error(),
        ("h".to_owned(), ErrorCode::InvalidRequest)
    );
    assert!(is_exit(control));
}

#[test]
fn firefox_and_safari_are_allowed_only_with_the_project_ids() {
    for (family, ours) in [
        (BrowserFamily::Firefox, websign_project::FIREFOX_ID),
        (
            BrowserFamily::Safari,
            websign_project::SAFARI_EXTENSION_BUNDLE_ID,
        ),
    ] {
        let launch = |id: &str| BrowserLaunch {
            origin: id.to_owned(),
            family,
            extension_id: id.to_owned(),
            parent_window: None,
        };
        let mut ok = Harness::new(
            Transport::NativeMessaging {
                launch: launch(ours),
            },
            (1, 1),
        );
        ok.send(wire::hello_native("h", 1, 1));
        assert_eq!(ok.take().kinds(), ["hello"], "{family}");

        let mut bad = Harness::new(
            Transport::NativeMessaging {
                launch: launch("org.example.other"),
            },
            (1, 1),
        );
        let control = bad.send(wire::hello_native("h", 1, 1));
        assert_eq!(
            bad.take().only_error().1,
            ErrorCode::InvalidRequest,
            "{family}"
        );
        assert!(is_exit(control));
    }
}

#[test]
fn a_frame_before_hello_is_refused_at_its_own_version() {
    let mut h = Harness::native();
    let mut frame: serde_json::Value =
        serde_json::from_slice(&wire::status("s", Some(ORIGIN))).unwrap();
    frame["v"] = serde_json::json!(7);
    h.send(serde_json::to_vec(&frame).unwrap());
    let out = h.take();
    assert_eq!(out.only_error().1, ErrorCode::InvalidRequest);
    assert_eq!(out.frames[0].v, 7);
}

#[test]
fn no_hello_within_five_seconds_ends_the_process() {
    let mut h = Harness::native();
    assert_eq!(h.pass(4), Control::Continue);
    assert_eq!(h.pass(1), Control::Exit(0));

    let mut ready = Harness::native_ready();
    assert_eq!(ready.pass(3_600), Control::Continue, "only before hello");
}
