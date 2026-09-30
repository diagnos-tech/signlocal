//! The app's answer to `hello` and the version of a refusal (SPEC §5.1).

mod common;

use common::*;
use serde_json::{Value, json};
use websign_protocol::messages::AppMessage;
use websign_protocol::{
    ErrorCode, PROTOCOL_VERSION, parse_app_message, parse_hello_reply, refusal_version,
};

fn refusal(v: u32, code: &str) -> Value {
    json!({ "v": v, "id": "h", "type": "error", "code": code, "message": "no" })
}

#[test]
fn accepts_the_apps_hello() {
    let envelope = parse_hello_reply(&to_bytes(&hello_reply()), 1).unwrap();
    assert!(matches!(envelope.message, AppMessage::Hello(_)));
}

#[test]
fn the_apps_hello_still_carries_the_chosen_protocol() {
    let reply = with(hello_reply(), "v", json!(2));
    assert_invalid(parse_hello_reply(&to_bytes(&reply), 2), id("h"));
}

#[test]
fn accepts_an_error_at_the_hello_version() {
    for (v, code) in [
        (1, "ClientOutdated"),
        (1, "InvalidRequest"),
        (7, "AppOutdated"),
    ] {
        let envelope = parse_hello_reply(&to_bytes(&refusal(v, code)), v).unwrap();
        assert_eq!(envelope.v, v);
        let AppMessage::Error(error) = envelope.message else {
            panic!("expected an error");
        };
        assert_eq!(serde_json::to_value(error.code).unwrap(), json!(code));
    }
}

#[test]
fn keeps_the_version_details_of_an_outdated_refusal() {
    let frame = with(
        refusal(1, "ClientOutdated"),
        "details",
        json!({ "installed": "1", "required": "2" }),
    );
    let envelope = parse_hello_reply(&to_bytes(&frame), 1).unwrap();
    let AppMessage::Error(error) = envelope.message else {
        panic!("expected an error");
    };
    assert_eq!(error.code, ErrorCode::ClientOutdated);
    assert_eq!(error.details.unwrap().required.as_deref(), Some("2"));
}

#[test]
fn refuses_an_error_at_another_version() {
    let error = assert_invalid(
        parse_hello_reply(&to_bytes(&refusal(2, "ClientOutdated")), 1),
        id("h"),
    );
    assert!(error.message.contains("version"), "{}", error.message);
}

#[test]
fn refuses_any_other_message() {
    let frame = with(need_digest(), "id", json!("h"));
    let error = assert_invalid(parse_hello_reply(&to_bytes(&frame), 1), id("h"));
    assert!(
        error.message.contains("hello or error"),
        "{}",
        error.message
    );
}

#[test]
fn the_refusal_body_is_as_strict_as_ever() {
    let frame = with(refusal(1, "InvalidRequest"), "extra", json!(1));
    assert_invalid(parse_hello_reply(&to_bytes(&frame), 1), id("h"));
    let frame = with(refusal(1, "InvalidRequest"), "code", json!("Nope"));
    assert_invalid(parse_hello_reply(&to_bytes(&frame), 1), id("h"));
}

#[test]
fn the_plain_parser_still_accepts_only_hello_first() {
    let frame = to_bytes(&refusal(1, "ClientOutdated"));
    assert_invalid(parse_app_message(&frame, None), id("h"));
}

#[test]
fn a_refusal_echoes_the_first_frames_version() {
    let hello_v3 = with(hello(), "v", json!(3));
    assert_eq!(refusal_version(&to_bytes(&hello_v3)), 3);
    assert_eq!(refusal_version(&to_bytes(&hello())), 1);
    let status_first = json!({ "v": 2, "id": "1", "type": "status" });
    assert_eq!(refusal_version(&to_bytes(&status_first)), 2);
}

#[test]
fn a_refusal_without_a_usable_version_uses_the_current_one() {
    let unusable = [
        json!(0),
        json!(-1),
        json!(1.5),
        json!("1"),
        json!(1u64 << 33),
    ];
    for v in unusable {
        let frame = with(hello(), "v", v);
        assert_eq!(refusal_version(&to_bytes(&frame)), PROTOCOL_VERSION);
    }
    assert_eq!(
        refusal_version(&to_bytes(&without(hello(), "v"))),
        PROTOCOL_VERSION
    );
    assert_eq!(refusal_version(b"not json"), PROTOCOL_VERSION);
}
