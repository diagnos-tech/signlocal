//! Session rules of `SPEC.md` §2, driven with hand-written frames.

use websign_core::present::caller::DesktopCaller;
use websign_protocol::types::{AppInfo, Channel, OsName};
use websign_protocol::{ClientMessage, ErrorCode, ProtocolRange, RequestId};
use websign_ui_model::confirm::port::RequestKey;

use super::*;
use crate::launch::BrowserLaunch;

fn app(min: u32, max: u32) -> AppInfo {
    AppInfo {
        version: "1.0.0".into(),
        protocols: ProtocolRange { min, max },
        os: OsName::Linux,
        arch: "x86_64".into(),
        channel: Channel::Direct,
    }
}

fn native() -> Session {
    Session::new(
        Transport::NativeMessaging {
            launch: BrowserLaunch::manual(),
        },
        app(1, 1),
    )
}

fn desktop() -> Session {
    Session::new(
        Transport::Desktop {
            caller: DesktopCaller {
                executable: "/usr/bin/tool".into(),
                product_name: None,
                signer: None,
            },
        },
        app(1, 1),
    )
}

const WEB: &str = r#""web":{"origin":"https://app.example","topOrigin":"https://app.example"}"#;

fn hello_frame(id: &str, min: u32, max: u32, v: u32, browser: bool) -> Vec<u8> {
    let browser = if browser {
        r#","browser":{"name":"chrome","version":"1","reason":"page"}"#
    } else {
        ""
    };
    format!(
        r#"{{"v":{v},"id":"{id}","type":"hello","client":{{"name":"t","version":"1"}},"protocols":{{"min":{min},"max":{max}}}{browser}}}"#
    )
    .into_bytes()
}

fn frame(id: &str, body: &str) -> Vec<u8> {
    format!(r#"{{"v":1,"id":"{id}",{body}}}"#).into_bytes()
}

fn native_ready() -> Session {
    let mut session = native();
    session
        .accept(&hello_frame("h", 1, 1, 1, true))
        .expect("hello");
    session
}

fn rejected(result: Result<Accepted, Box<Rejection>>) -> Box<Rejection> {
    result.expect_err("frame must be refused")
}

#[test]
fn hello_negotiates_and_stores_the_version() {
    let mut session = native();
    let accepted = session.accept(&hello_frame("h", 1, 3, 3, true)).unwrap();
    assert!(accepted.caller.is_none());
    assert_eq!(session.negotiated(), Some(1));
}

#[test]
fn version_mismatch_in_either_direction_closes() {
    let mut newer_app = Session::new(
        Transport::NativeMessaging {
            launch: BrowserLaunch::manual(),
        },
        app(3, 4),
    );
    let old_client = rejected(newer_app.accept(&hello_frame("h", 1, 2, 2, true)));
    assert_eq!(old_client.error.code, ErrorCode::ClientOutdated);
    assert!(old_client.close);

    let mut older_app = native();
    let new_client = rejected(older_app.accept(&hello_frame("h", 2, 3, 3, true)));
    assert_eq!(new_client.error.code, ErrorCode::AppOutdated);
    assert!(new_client.close);
    assert_eq!(older_app.negotiated(), None);
}

#[test]
fn anything_before_hello_is_refused_and_closes() {
    let mut session = native();
    let refusal = rejected(session.accept(&frame("1", r#""type":"status""#)));
    assert_eq!(refusal.error.code, ErrorCode::InvalidRequest);
    assert_eq!(refusal.id.as_ref().map(RequestId::as_str), Some("1"));
    assert!(refusal.close);
}

#[test]
fn a_frame_without_an_id_closes() {
    let mut session = native_ready();
    let refusal = rejected(session.accept(br#"{"v":1,"type":"status"}"#));
    assert!(refusal.id.is_none());
    assert!(refusal.close);
}

#[test]
fn a_second_hello_is_refused_but_the_connection_stays() {
    let mut session = native_ready();
    let refusal = rejected(session.accept(&hello_frame("again", 1, 1, 1, true)));
    assert_eq!(refusal.error.code, ErrorCode::InvalidRequest);
    assert!(!refusal.close);
    assert_eq!(session.negotiated(), Some(1));
}

#[test]
fn native_messaging_hello_needs_browser_and_desktop_refuses_it() {
    let refusal = rejected(native().accept(&hello_frame("h", 1, 1, 1, false)));
    assert!(refusal.error.message.contains("browser"));
    assert!(refusal.close);
    let refusal = rejected(desktop().accept(&hello_frame("h", 1, 1, 1, true)));
    assert!(refusal.error.message.contains("browser"));
}

#[test]
fn requests_carry_the_caller_of_their_transport() {
    let mut web = native_ready();
    let accepted = web
        .accept(&frame("1", &format!(r#""type":"status",{WEB}"#)))
        .unwrap();
    assert!(matches!(accepted.caller, Some(Caller::Web { .. })));

    let mut program = desktop();
    program.accept(&hello_frame("h", 1, 1, 1, false)).unwrap();
    let accepted = program.accept(&frame("1", r#""type":"status""#)).unwrap();
    assert!(matches!(accepted.caller, Some(Caller::Desktop(_))));
}

#[test]
fn web_context_rules_follow_the_transport() {
    let mut web = native_ready();
    let refusal = rejected(web.accept(&frame("1", r#""type":"status""#)));
    assert_eq!(refusal.error.code, ErrorCode::InvalidRequest);
    assert!(!refusal.close);

    let mut program = desktop();
    program.accept(&hello_frame("h", 1, 1, 1, false)).unwrap();
    let refusal = rejected(program.accept(&frame("1", &format!(r#""type":"status",{WEB}"#))));
    assert_eq!(refusal.error.code, ErrorCode::InvalidRequest);
}

#[test]
fn insecure_origins_are_refused_by_code() {
    let mut session = native_ready();
    let body = r#""type":"status","web":{"origin":"http://evil.example","topOrigin":"http://evil.example"}"#;
    let refusal = rejected(session.accept(&frame("1", body)));
    assert_eq!(refusal.error.code, ErrorCode::InsecureOrigin);
}

#[test]
fn duplicate_ids_and_the_in_flight_limit_are_enforced() {
    let mut session = native_ready();
    let begin = |id: &str| frame(id, &format!(r#""type":"choose",{WEB}"#));
    let id = RequestId::new("1").unwrap();
    session.open(id.clone(), RequestKey(1));
    let duplicate = rejected(session.accept(&begin("1")));
    assert_eq!(duplicate.error.message, "duplicate id");

    for n in 2..=16 {
        session.open(RequestId::new(format!("r{n}")).unwrap(), RequestKey(n));
    }
    let busy = rejected(session.accept(&begin("late")));
    assert_eq!(busy.error.code, ErrorCode::Busy);
    session.close(&id);
    assert!(session.accept(&begin("late")).is_ok());
}

#[test]
fn continuations_need_an_open_id_except_cancel() {
    let mut session = native_ready();
    let digest = frame("9", r#""type":"sign.digest","seq":1,"digest":"AAAA""#);
    let refusal = rejected(session.accept(&digest));
    assert_eq!(refusal.error.code, ErrorCode::InvalidRequest);
    assert!(!refusal.close);

    let cancel = session.accept(&frame("9", r#""type":"cancel""#)).unwrap();
    assert!(matches!(cancel.envelope.message, ClientMessage::Cancel(_)));

    session.open(RequestId::new("9").unwrap(), RequestKey(4));
    assert!(session.accept(&digest).is_ok());
}

#[test]
fn unknown_fields_are_refused_with_the_request_id() {
    let mut session = native_ready();
    let body = format!(r#""type":"status",{WEB},"extra":1"#);
    let refusal = rejected(session.accept(&frame("7", &body)));
    assert_eq!(refusal.id.as_ref().map(RequestId::as_str), Some("7"));
    assert!(!refusal.close);
}
