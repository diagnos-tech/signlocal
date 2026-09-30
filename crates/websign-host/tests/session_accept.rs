//! SPEC §2: `Session::accept` and the identity of the caller.

mod common;

use common::harness::{allowed_launch, app_info, desktop_caller};
use common::{ORIGIN, wire};
use websign_host::Caller;
use websign_host::session::{Session, Transport};
use websign_protocol::limits::MAX_IN_FLIGHT_PER_CONNECTION;
use websign_protocol::{ClientMessage, ErrorCode, RequestId};
use websign_ui_model::confirm::port::RequestKey;

fn native() -> Session {
    Session::new(
        Transport::NativeMessaging {
            launch: allowed_launch(),
        },
        app_info(1, 1),
    )
}

fn desktop() -> Session {
    Session::new(
        Transport::Desktop {
            caller: desktop_caller(),
        },
        app_info(1, 1),
    )
}

fn ready(mut session: Session, hello: Vec<u8>) -> Session {
    session.accept(&hello).expect("hello accepted");
    session
}

fn native_ready() -> Session {
    ready(native(), wire::hello_native("h", 1, 1))
}

fn id(text: &str) -> RequestId {
    RequestId::new(text).expect("valid id")
}

fn refusal(session: &mut Session, frame: Vec<u8>) -> (Option<String>, ErrorCode, bool) {
    let rejection = session.accept(&frame).expect_err("refused");
    (
        rejection.id.map(|i| i.to_string()),
        rejection.error.code,
        rejection.close,
    )
}

#[test]
fn hello_negotiates_stores_the_version_and_has_no_caller() {
    let mut session = native();
    assert_eq!(session.negotiated(), None);
    let accepted = session
        .accept(&wire::hello_native("h", 1, 1))
        .expect("accepted");
    assert!(matches!(accepted.envelope.message, ClientMessage::Hello(_)));
    assert!(accepted.caller.is_none());
    assert_eq!(session.negotiated(), Some(1));
}

#[test]
fn a_failed_negotiation_closes_and_leaves_the_session_unnegotiated() {
    let mut session = native();
    let (id, code, close) = refusal(&mut session, wire::hello_native("h", 2, 3));
    assert_eq!(
        (id.as_deref(), code, close),
        (Some("h"), ErrorCode::AppOutdated, true)
    );
    assert_eq!(session.negotiated(), None);
}

#[test]
fn before_hello_only_hello_is_accepted_and_the_stream_closes() {
    let mut session = native();
    let (id, code, close) = refusal(&mut session, wire::status("s", Some(ORIGIN)));
    assert_eq!(
        (id.as_deref(), code, close),
        (Some("s"), ErrorCode::InvalidRequest, true)
    );
}

#[test]
fn a_frame_without_an_id_closes() {
    let mut session = native_ready();
    let (id, _, close) = refusal(&mut session, b"{\"type\":\"cancel\"}".to_vec());
    assert_eq!(id, None);
    assert!(
        close,
        "no id means nobody to answer and no trust in the stream"
    );
}

#[test]
fn a_parse_error_after_hello_answers_the_id_and_keeps_the_stream() {
    let mut session = native_ready();
    let raw = br#"{"v":1,"id":"x","type":"sign.begin","hash":"SHA-1","web":{"origin":"https://a.example","topOrigin":"https://a.example"}}"#;
    let (id, code, close) = refusal(&mut session, raw.to_vec());
    assert_eq!(
        (id.as_deref(), code, close),
        (Some("x"), ErrorCode::InvalidRequest, false)
    );
}

#[test]
fn a_second_hello_is_invalid_but_does_not_close() {
    let mut session = native_ready();
    let (id, code, close) = refusal(&mut session, wire::hello_native("again", 1, 1));
    assert_eq!(
        (id.as_deref(), code, close),
        (Some("again"), ErrorCode::InvalidRequest, false)
    );
    assert_eq!(session.negotiated(), Some(1), "the agreement stands");
}

#[test]
fn web_requests_get_a_web_caller_built_from_the_transport() {
    let mut session = native_ready();
    let accepted = session
        .accept(&wire::sign_begin("s", Some(ORIGIN), "SHA-256"))
        .expect("ok");
    match accepted.caller {
        Some(Caller::Web {
            origin,
            top,
            browser,
        }) => {
            assert_eq!(origin.canonical, ORIGIN);
            assert!(top.is_none());
            assert_eq!(browser.version, "129.0");
        }
        other => panic!("expected a web caller, got {other:?}"),
    }
}

#[test]
fn desktop_requests_get_the_transports_caller() {
    let mut session = ready(desktop(), wire::hello_desktop("h", 1, 1));
    let accepted = session
        .accept(&wire::sign_begin("s", None, "SHA-256"))
        .expect("ok");
    assert_eq!(accepted.caller, Some(Caller::Desktop(desktop_caller())));
}

#[test]
fn insecure_and_malformed_origins_map_to_their_codes() {
    let mut session = native_ready();
    let bad = wire::sign_begin("a", Some("http://evil.example"), "SHA-256");
    assert_eq!(refusal(&mut session, bad).1, ErrorCode::InsecureOrigin);
    let bad = wire::sign_begin("b", Some("nonsense"), "SHA-256");
    assert_eq!(refusal(&mut session, bad).1, ErrorCode::InvalidRequest);
}

#[test]
fn a_duplicate_request_id_is_refused_while_the_first_is_open() {
    let mut session = native_ready();
    session
        .accept(&wire::sign_begin("s", Some(ORIGIN), "SHA-256"))
        .expect("first");
    session.open(id("s"), RequestKey(1));
    let (_, code, close) = refusal(&mut session, wire::sign_begin("s", Some(ORIGIN), "SHA-256"));
    assert_eq!((code, close), (ErrorCode::InvalidRequest, false));

    session.close(&id("s"));
    session
        .accept(&wire::sign_begin("s", Some(ORIGIN), "SHA-256"))
        .expect("free again");
}

#[test]
fn open_lookup_and_close_track_request_keys() {
    let mut session = native_ready();
    assert_eq!(session.lookup(&id("a")), None);
    session.open(id("a"), RequestKey(7));
    assert_eq!(session.lookup(&id("a")), Some(RequestKey(7)));
    assert_eq!(session.close(&id("a")), Some(RequestKey(7)));
    assert_eq!(session.lookup(&id("a")), None);
    assert_eq!(session.close(&id("a")), None);
}

#[test]
fn the_sixteenth_open_request_is_the_last() {
    let mut session = native_ready();
    for n in 0..MAX_IN_FLIGHT_PER_CONNECTION {
        session.open(id(&format!("r{n}")), RequestKey(n as u64));
    }
    let (i, code, close) = refusal(
        &mut session,
        wire::sign_begin("over", Some(ORIGIN), "SHA-256"),
    );
    assert_eq!(
        (i.as_deref(), code, close),
        (Some("over"), ErrorCode::Busy, false)
    );
    session.close(&id("r3"));
    session
        .accept(&wire::sign_begin("over", Some(ORIGIN), "SHA-256"))
        .expect("room again");
}

#[test]
fn continuations_need_an_open_request_except_cancel() {
    let mut session = native_ready();
    let (i, code, _) = refusal(&mut session, wire::digest("gone", 1, &[0; 32]));
    assert_eq!(
        (i.as_deref(), code),
        (Some("gone"), ErrorCode::InvalidRequest)
    );

    let accepted = session
        .accept(&wire::cancel("gone"))
        .expect("cancel is always accepted");
    assert!(matches!(
        accepted.envelope.message,
        ClientMessage::Cancel(_)
    ));

    session.open(id("s"), RequestKey(1));
    let accepted = session
        .accept(&wire::digest("s", 1, &[0; 32]))
        .expect("open id");
    assert!(matches!(
        accepted.envelope.message,
        ClientMessage::SignDigest(_)
    ));
    assert!(accepted.caller.is_none(), "continuations carry no caller");
}

#[test]
fn the_in_flight_limit_does_not_apply_to_continuations() {
    let mut session = native_ready();
    for n in 0..MAX_IN_FLIGHT_PER_CONNECTION {
        session.open(id(&format!("r{n}")), RequestKey(n as u64));
    }
    session.accept(&wire::cancel("r1")).expect("cancel");
    session
        .accept(&wire::digest("r2", 1, &[0; 32]))
        .expect("digest");
}
