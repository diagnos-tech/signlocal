use super::*;
use crate::messages::{Cancel, Done, SignBegin, SignDigest, Status};
use crate::types::{Base64Bytes, HashName};

fn client(frame: &str, negotiated: Option<u32>) -> Result<ClientEnvelope, ParseError> {
    parse_client_message(frame.as_bytes(), negotiated)
}

fn error_of(frame: &str, negotiated: Option<u32>) -> ParseError {
    client(frame, negotiated).expect_err("frame must be refused")
}

const HELLO: &str = r#"{"v":1,"id":"h","type":"hello","client":{"name":"c","version":"1"},"protocols":{"min":1,"max":1}}"#;

#[test]
fn parses_hello_before_negotiation() {
    let envelope = client(HELLO, None).expect("valid hello");
    assert_eq!(envelope.message.kind(), "hello");
    assert_eq!(envelope.id.as_str(), "h");
}

#[test]
fn anything_but_hello_is_refused_first() {
    let error = error_of(r#"{"v":1,"id":"1","type":"status"}"#, None);
    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(error.id.as_ref().map(RequestId::as_str), Some("1"));
    assert!(error.message.contains("hello must be the first message"));
}

#[test]
fn hello_version_must_be_offered_by_its_own_range() {
    let frame = HELLO.replace(r#""v":1"#, r#""v":2"#);
    assert_eq!(error_of(&frame, None).code, ErrorCode::InvalidRequest);
    let frame = HELLO.replace(r#""v":1"#, r#""v":0"#);
    assert_eq!(error_of(&frame, None).code, ErrorCode::InvalidRequest);
}

#[test]
fn later_messages_must_match_the_negotiated_version() {
    let frame = r#"{"v":2,"id":"1","type":"status"}"#;
    let error = error_of(frame, Some(1));
    assert!(
        error
            .message
            .contains("does not match the negotiated version 1")
    );
    assert!(client(r#"{"v":1,"id":"1","type":"status"}"#, Some(1)).is_ok());
}

#[test]
fn unreadable_frames_carry_no_id() {
    for frame in [
        "",
        "[]",
        "7",
        "{",
        r#"{"v":1,"type":"status"}"#,
        r#"{"v":1,"id":7,"type":"status"}"#,
        r#"{"v":1,"id":"a b","type":"status"}"#,
    ] {
        let error = error_of(frame, Some(1));
        assert_eq!(
            (error.id, error.code),
            (None, ErrorCode::InvalidRequest),
            "{frame}"
        );
    }
    let error = parse_client_message(&[0xff, 0xfe], Some(1)).expect_err("not UTF-8");
    assert_eq!(error.id, None);
}

#[test]
fn envelope_field_errors_keep_the_id() {
    for frame in [
        r#"{"id":"1","type":"status"}"#,
        r#"{"v":"1","id":"1","type":"status"}"#,
        r#"{"v":-1,"id":"1","type":"status"}"#,
        r#"{"v":1.5,"id":"1","type":"status"}"#,
        r#"{"v":1,"id":"1"}"#,
        r#"{"v":1,"id":"1","type":7}"#,
        r#"{"v":1,"id":"1","type":"nope"}"#,
        r#"{"v":1,"id":"1","type":"choose.result"}"#,
    ] {
        let error = error_of(frame, Some(1));
        assert_eq!(
            error.id.as_ref().map(RequestId::as_str),
            Some("1"),
            "{frame}"
        );
    }
}

#[test]
fn unknown_type_names_are_echoed_only_when_short_and_printable() {
    let short = error_of(r#"{"v":1,"id":"1","type":"nope"}"#, Some(1));
    assert!(short.message.contains("nope"));
    let long = format!(r#"{{"v":1,"id":"1","type":"{}"}}"#, "x".repeat(33));
    assert!(!error_of(&long, Some(1)).message.contains("xxx"));
    let control = error_of("{\"v\":1,\"id\":\"1\",\"type\":\"a\\nb\"}", Some(1));
    assert!(!control.message.contains('\n'));
}

#[test]
fn unknown_wrong_typed_and_invalid_fields_are_refused_by_name() {
    let cases = [
        (r#"{"v":1,"id":"1","type":"status","extra":1}"#, "extra"),
        (
            r#"{"v":1,"id":"1","type":"sign.begin","hash":"MD5"}"#,
            "hash",
        ),
        (
            r#"{"v":1,"id":"1","type":"sign.begin","hash":"SHA-256","algorithms":"ECDSA"}"#,
            "algorithms",
        ),
        (
            r#"{"v":1,"id":"1","type":"sign.begin","hash":"SHA-256","certificate":"ABC"}"#,
            "certificate",
        ),
        (
            r#"{"v":1,"id":"1","type":"sign.digest","seq":1,"digest":"not base64!"}"#,
            "digest",
        ),
        (r#"{"v":1,"id":"1","type":"sign.begin"}"#, "hash"),
        (r#"{"v":1,"id":"1","type":"cancel","reason":"x"}"#, "reason"),
    ];
    for (frame, field) in cases {
        let error = error_of(frame, Some(1));
        assert_eq!(error.code, ErrorCode::InvalidRequest, "{frame}");
        assert!(error.message.contains(field), "{frame}: {}", error.message);
    }
}

#[test]
fn error_messages_never_echo_values() {
    let error = error_of(
        r#"{"v":1,"id":"1","type":"sign.begin","hash":"SECRET-VALUE"}"#,
        Some(1),
    );
    assert!(!error.message.contains("SECRET"));
    let error = error_of(
        r#"{"v":1,"id":"1","type":"sign.digest","seq":1,"digest":"SECRETSECRET!"}"#,
        Some(1),
    );
    assert!(!error.message.contains("SECRET"));
}

#[test]
fn hostile_field_names_are_not_echoed() {
    let error = error_of(
        "{\"v\":1,\"id\":\"1\",\"type\":\"status\",\"a\\u001b[31m\":1}",
        Some(1),
    );
    assert!(!error.message.contains('\u{1b}'));
}

#[test]
fn app_messages_parse_with_their_own_catalog() {
    let done = parse_app_message(br#"{"v":1,"id":"1","type":"done"}"#, Some(1)).expect("done");
    assert_eq!(done.message, AppMessage::Done(Done {}));
    let refused = parse_app_message(
        br#"{"v":1,"id":"1","type":"sign.begin","hash":"SHA-256"}"#,
        Some(1),
    )
    .expect_err("client-only type");
    assert_eq!(refused.code, ErrorCode::InvalidRequest);
}

#[test]
fn every_client_message_round_trips_compactly() {
    let messages = vec![
        ClientMessage::Status(Status::default()),
        ClientMessage::SignBegin(SignBegin {
            web: None,
            hash: HashName::Sha384,
            algorithms: None,
            certificate: None,
        }),
        ClientMessage::SignDigest(SignDigest {
            seq: 2,
            digest: Base64Bytes::new(vec![7; 48]),
        }),
        ClientMessage::Cancel(Cancel {}),
    ];
    for message in messages {
        let envelope = ClientEnvelope {
            v: 1,
            id: RequestId::new("r.1").expect("valid id"),
            message,
        };
        let bytes = to_json(&envelope);
        assert!(!bytes.contains(&b' ') && !bytes.contains(&b'\n'));
        assert_eq!(parse_client_message(&bytes, Some(1)), Ok(envelope));
    }
}

#[test]
fn app_errors_round_trip() {
    for code in ErrorCode::ALL {
        let envelope = AppEnvelope {
            v: 1,
            id: RequestId::new("e").expect("valid id"),
            message: AppMessage::Error(crate::error::WireError {
                code,
                message: "m".into(),
                details: None,
            }),
        };
        assert_eq!(
            parse_app_message(&to_json(&envelope), Some(1)),
            Ok(envelope)
        );
    }
}
