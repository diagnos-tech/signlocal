//! Envelope parsing: the check order of SPEC §5, steps 1 to 5.

mod common;

use common::*;
use serde_json::json;
use websign_protocol::ParseError;

fn client_bytes(
    bytes: &[u8],
    negotiated: Option<u32>,
) -> Result<websign_protocol::ClientEnvelope, ParseError> {
    websign_protocol::parse_client_message(bytes, negotiated)
}

// ---- 1. not JSON / not an object -------------------------------------

#[test]
fn non_json_frames_have_no_id() {
    let frames: [&[u8]; 9] = [
        b"",
        b"   ",
        b"not json",
        b"[]",
        b"\"x\"",
        b"1",
        b"null",
        b"true",
        br#"{"v":1,"id":"1","type":"cancel"} trailing"#,
    ];
    for frame in frames {
        assert_invalid(client_bytes(frame, Some(1)), None);
        assert_invalid(websign_protocol::parse_app_message(frame, Some(1)), None);
    }
}

#[test]
fn invalid_utf8_has_no_id() {
    let mut frame = br#"{"v":1,"id":"1","type":"status","web":{"origin":""#.to_vec();
    frame.extend_from_slice(&[0xff, 0xfe]);
    frame.extend_from_slice(br#"","topOrigin":"x"}}"#);
    assert_invalid(client_bytes(&frame, Some(1)), None);
    assert_invalid(client_bytes(&[0xff, 0xfe, 0xfd], None), None);
}

// ---- 2. id -----------------------------------------------------------

#[test]
fn bad_id_reports_no_id() {
    let bad = [
        json!(null),
        json!(7),
        json!(true),
        json!([]),
        json!({}),
        json!(""),
        json!("a b"),
        json!("é"),
        json!("a\n"),
        json!("<x>"),
        json!("a".repeat(65)),
    ];
    for value in bad {
        assert_invalid(
            parse_client(&with(sign_begin(), "id", value.clone()), Some(1)),
            None,
        );
    }
    assert_invalid(parse_client(&without(sign_begin(), "id"), Some(1)), None);
}

#[test]
fn id_is_checked_before_everything_else() {
    let frame = json!({ "id": "a b", "type": "nope" });
    assert_invalid(parse_client(&frame, Some(1)), None);
}

#[test]
fn max_length_id_is_accepted_and_echoed() {
    let long = "a".repeat(64);
    let envelope = parse_client(&with(sign_begin(), "id", json!(long)), Some(1)).unwrap();
    assert_eq!(envelope.id.as_str(), long);
}

// ---- 3. v ------------------------------------------------------------

#[test]
fn bad_v_reports_the_id() {
    let bad = [
        json!(null),
        json!("1"),
        json!(0),
        json!(-1),
        json!(1.5),
        json!(true),
        json!([1]),
        json!(4_294_967_296u64),
    ];
    for value in bad {
        assert_invalid(
            parse_client(&with(sign_begin(), "v", value), Some(1)),
            id("3"),
        );
    }
    assert_invalid(parse_client(&without(sign_begin(), "v"), Some(1)), id("3"));
}

// ---- 4. type ---------------------------------------------------------

#[test]
fn bad_type_reports_the_id() {
    for value in [
        json!(null),
        json!(1),
        json!("nope"),
        json!(""),
        json!("Status"),
        json!("sign.begin "),
    ] {
        assert_invalid(
            parse_client(&with(sign_begin(), "type", value), Some(1)),
            id("3"),
        );
    }
    assert_invalid(
        parse_client(&without(sign_begin(), "type"), Some(1)),
        id("3"),
    );
}

#[test]
fn types_are_directional() {
    for app_only in [need_digest(), sign_result()] {
        assert_invalid(parse_client(&app_only, Some(1)), id("3"));
    }
    let choose_result =
        json!({ "v": 1, "id": "2", "type": "choose.result", "certificates": [certificate()] });
    assert_invalid(parse_client(&choose_result, Some(1)), id("2"));
    for client_only in [
        sign_begin(),
        sign_digest(),
        json!({ "v": 1, "id": "3", "type": "cancel" }),
        json!({ "v": 1, "id": "3", "type": "diagnostics.open" }),
    ] {
        assert_invalid(parse_app(&client_only, Some(1)), id("3"));
    }
}

#[test]
fn unknown_type_is_named_only_when_short_printable_ascii() {
    let error = assert_invalid(
        parse_client(&with(sign_begin(), "type", json!("bogus.type")), Some(1)),
        id("3"),
    );
    assert!(error.message.contains("bogus.type"), "{}", error.message);

    let exactly_32 = "x".repeat(32);
    let error = assert_invalid(
        parse_client(&with(sign_begin(), "type", json!(exactly_32)), Some(1)),
        id("3"),
    );
    assert!(error.message.contains(&exactly_32), "{}", error.message);

    for hidden in [
        "y".repeat(33),
        "tipo-é".to_string(),
        "bad\u{7}type".to_string(),
        "tab\there".to_string(),
    ] {
        let error = assert_invalid(
            parse_client(&with(sign_begin(), "type", json!(hidden)), Some(1)),
            id("3"),
        );
        assert!(!error.message.contains(&hidden), "{}", error.message);
    }
}

// ---- 5. version ------------------------------------------------------

#[test]
fn before_negotiation_only_hello_is_accepted() {
    for frame in [
        sign_begin(),
        sign_digest(),
        json!({ "v": 1, "id": "1", "type": "status" }),
        json!({ "v": 1, "id": "1", "type": "cancel" }),
    ] {
        let error = assert_invalid(
            parse_client(&frame, None),
            frame["id"].as_str().and_then(id),
        );
        assert!(error.message.contains("hello"), "{}", error.message);
    }
}

#[test]
fn hello_is_not_rejected_for_v_alone_but_must_be_in_its_own_range() {
    let mut frame = hello();
    frame["v"] = json!(2);
    frame["protocols"] = json!({ "min": 1, "max": 3 });
    assert!(parse_client(&frame, None).is_ok());

    frame["protocols"] = json!({ "min": 3, "max": 4 });
    assert_invalid(parse_client(&frame, None), id("h"));

    frame["v"] = json!(4);
    frame["protocols"] = json!({ "min": 1, "max": 3 });
    assert_invalid(parse_client(&frame, None), id("h"));
}

#[test]
fn after_negotiation_v_must_equal_the_negotiated_version() {
    assert!(parse_client(&sign_begin(), Some(1)).is_ok());
    for negotiated in [2, 3] {
        let error = assert_invalid(parse_client(&sign_begin(), Some(negotiated)), id("3"));
        assert!(error.message.contains("version"), "{}", error.message);
    }
    let mut frame = sign_begin();
    frame["v"] = json!(2);
    assert_invalid(parse_client(&frame, Some(1)), id("3"));
    assert!(parse_client(&frame, Some(2)).is_ok());
}

#[test]
fn version_is_checked_before_the_body() {
    let broken = with(with(sign_begin(), "v", json!(2)), "hash", json!("MD5"));
    let error = assert_invalid(parse_client(&broken, Some(1)), id("3"));
    assert!(error.message.contains("version"), "{}", error.message);
}

#[test]
fn app_messages_follow_the_same_version_rules() {
    assert!(parse_app(&need_digest(), Some(1)).is_ok());
    assert_invalid(parse_app(&need_digest(), Some(2)), id("3"));
    assert_invalid(parse_app(&need_digest(), None), id("3"));
    assert!(parse_app(&hello_reply(), None).is_ok());
}
