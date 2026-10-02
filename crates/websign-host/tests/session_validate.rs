//! SPEC §2.2: the per-transport table, message by message.

mod common;

use common::ORIGIN;
use common::harness::{allowed_launch, desktop_caller};
use serde_json::json;
use websign_host::session::{Transport, validate};
use websign_protocol::messages::Hello;
use websign_protocol::types::{BrowserInfo, BrowserName, ClientInfo, HelloReason};
use websign_protocol::{ClientMessage, ErrorCode, ProtocolRange};

fn native() -> Transport {
    Transport::NativeMessaging {
        launch: allowed_launch(),
    }
}

fn desktop() -> Transport {
    Transport::Desktop {
        caller: desktop_caller(),
    }
}

fn parse(kind: &str, extra: serde_json::Value) -> ClientMessage {
    let mut object = json!({ "v": 1, "id": "x", "type": kind });
    for (k, v) in extra.as_object().expect("object") {
        object[k] = v.clone();
    }
    let bytes = serde_json::to_vec(&object).expect("json");
    websign_protocol::parse_client_message(&bytes, Some(1))
        .expect("parses")
        .message
}

fn web() -> serde_json::Value {
    json!({ "web": { "origin": ORIGIN, "topOrigin": ORIGIN } })
}

fn hello(browser: bool) -> ClientMessage {
    ClientMessage::Hello(Hello {
        client: ClientInfo {
            name: "c".to_owned(),
            version: "1".to_owned(),
        },
        protocols: ProtocolRange { min: 1, max: 1 },
        browser: browser.then(|| BrowserInfo {
            name: BrowserName::Chrome,
            version: "129.0".to_owned(),
            reason: HelloReason::Page,
        }),
    })
}

#[test]
fn hello_needs_browser_on_native_messaging_and_refuses_it_on_desktop() {
    assert!(validate(&native(), &hello(true)).is_ok());
    assert_eq!(
        validate(&native(), &hello(false))
            .expect_err("refused")
            .code,
        ErrorCode::InvalidRequest
    );
    assert!(validate(&desktop(), &hello(false)).is_ok());
    assert_eq!(
        validate(&desktop(), &hello(true))
            .expect_err("refused")
            .code,
        ErrorCode::InvalidRequest
    );
}

#[test]
fn web_is_required_on_native_and_refused_on_desktop_for_the_three_requests() {
    let with_web = [
        parse("status", web()),
        parse("choose", web()),
        parse("sign.begin", {
            let mut v = web();
            v["hash"] = json!("SHA-256");
            v
        }),
    ];
    let without_web = [
        parse("status", json!({})),
        parse("choose", json!({})),
        parse("sign.begin", json!({ "hash": "SHA-256" })),
    ];
    for message in &with_web {
        assert!(validate(&native(), message).is_ok(), "{}", message.kind());
        assert!(validate(&desktop(), message).is_err(), "{}", message.kind());
    }
    for message in &without_web {
        assert!(validate(&native(), message).is_err(), "{}", message.kind());
        assert!(validate(&desktop(), message).is_ok(), "{}", message.kind());
    }
}

#[test]
fn diagnostics_open_and_the_continuations_are_allowed_on_both() {
    let messages = [
        parse("diagnostics.open", json!({})),
        parse("diagnostics.open", json!({ "tab": "help" })),
        parse("cancel", json!({})),
        parse(
            "sign.digest",
            json!({ "seq": 1, "digest": websign_protocol::base64::encode(&[0u8; 32]) }),
        ),
    ];
    for message in &messages {
        assert!(validate(&native(), message).is_ok(), "{}", message.kind());
        assert!(validate(&desktop(), message).is_ok(), "{}", message.kind());
    }
}

#[test]
fn refusals_are_invalid_request_and_name_the_field() {
    let err = validate(&native(), &parse("status", json!({}))).expect_err("refused");
    assert_eq!(err.code, ErrorCode::InvalidRequest);
    assert!(err.message.contains("web"), "{}", err.message);

    let err = validate(&desktop(), &parse("status", web())).expect_err("refused");
    assert!(err.message.contains("web"), "{}", err.message);

    let err = validate(&native(), &hello(false)).expect_err("refused");
    assert!(err.message.contains("browser"), "{}", err.message);
}

#[test]
fn the_message_never_echoes_the_origin_the_caller_sent() {
    let err = validate(&desktop(), &parse("status", web())).expect_err("refused");
    assert!(!err.message.contains("example.com"));
}
