//! One spelling per message (SPEC §1): repeated keys, `null` for absent
//! fields, arrays for objects and floats for integers are all refused, so a
//! relay and the app can never read the same bytes differently.

mod common;

use common::*;
use serde_json::{Value, json};
use websign_protocol::page::{PageRequest, PageStatus, PageToExtension};
use websign_protocol::types::Certificate;
use websign_protocol::version::ProtocolRange;

#[test]
fn repeated_keys_make_the_frame_unreadable() {
    for frame in [
        r#"{"v":1,"id":"a","id":"b","type":"status"}"#,
        r#"{"v":1,"id":"a","type":"status","type":"status"}"#,
        r#"{"v":1,"v":1,"id":"a","type":"status"}"#,
        r#"{"v":1,"id":"a","id":"a","type":"status"}"#,
        r#"{"v":1,"id":"3","type":"sign.begin","hash":"SHA-256","web":{"origin":"https://a.example","origin":"https://b.example","topOrigin":"https://a.example"}}"#,
        r#"{"v":1,"id":"3","type":"sign.begin","hash":"SHA-256","hash":"SHA-512"}"#,
    ] {
        let result = websign_protocol::parse_client_message(frame.as_bytes(), Some(1));
        assert_invalid(result, None);
    }
}

#[test]
fn optional_fields_are_absent_never_null() {
    let client_cases = [
        (with(sign_begin(), "web", Value::Null), "web"),
        (with(sign_begin(), "algorithms", Value::Null), "algorithms"),
        (
            with(sign_begin(), "certificate", Value::Null),
            "certificate",
        ),
        (
            json!({ "v": 1, "id": "2", "type": "choose", "web": web(), "filter": null }),
            "filter",
        ),
        (
            json!({ "v": 1, "id": "2", "type": "choose", "filter": { "algorithms": null } }),
            "algorithms",
        ),
        (
            json!({ "v": 1, "id": "d", "type": "diagnostics.open", "tab": null }),
            "tab",
        ),
    ];
    for (frame, field) in client_cases {
        assert_names_field_not_value(frame, field, "null");
    }
    let mut frame = hello();
    frame["browser"] = Value::Null;
    assert_invalid(parse_client(&frame, None), id("h"));

    let error = json!({ "v": 1, "id": "3", "type": "error", "code": "Busy", "message": "m" });
    assert_invalid(
        parse_app(&with(error.clone(), "details", Value::Null), Some(1)),
        id("3"),
    );
    let details = json!({ "installed": null });
    assert_invalid(
        parse_app(&with(error, "details", details), Some(1)),
        id("3"),
    );
    for (field, value) in [("icpBrasil", Value::Null), ("eidas", Value::Null)] {
        let mut certificate = certificate();
        certificate["profile"][field] = value;
        let frame = with(need_digest(), "certificate", certificate);
        assert_invalid(parse_app(&frame, Some(1)), id("3"));
    }

    let page = json!({ "type": "choose", "filter": null });
    assert!(serde_json::from_value::<PageRequest>(page).is_err());
    let status = json!({
        "extension": { "version": "1", "browser": "chrome" },
        "app": null, "appOutdated": false, "remembered": false
    });
    assert!(serde_json::from_value::<PageStatus>(status).is_err());
}

#[test]
fn objects_are_never_spelled_as_arrays() {
    assert!(serde_json::from_value::<ProtocolRange>(json!([1, 1])).is_err());
    let mut frame = hello();
    frame["protocols"] = json!([1, 1]);
    assert_invalid(parse_client(&frame, None), id("h"));
    assert_names_field_not_value(
        with(
            sign_begin(),
            "web",
            json!(["https://a.example", "https://a.example"]),
        ),
        "web",
        "https://a.example",
    );
    assert_names_field_not_value(
        json!({ "v": 1, "id": "2", "type": "choose", "filter": [] }),
        "filter",
        "[]",
    );

    let mut certificate = certificate();
    certificate["key"] = json!(["RSA", 2048]);
    assert!(serde_json::from_value::<Certificate>(certificate).is_err());
    let page =
        json!({ "source": "websign-page", "kind": "request", "id": "p1", "message": ["cancel"] });
    assert!(serde_json::from_value::<PageToExtension>(page).is_err());
    assert!(
        serde_json::from_value::<PageToExtension>(json!(["discover", "websign-page"])).is_err()
    );
}

#[test]
fn integers_are_never_spelled_as_floats() {
    for v in [json!(1.0), json!(1e0), json!("1")] {
        let frame = json!({ "v": v, "id": "1", "type": "status" });
        assert_invalid(parse_client(&frame, Some(1)), id("1"));
    }
    let raw = br#"{"v":1.0,"id":"1","type":"status"}"#;
    assert_invalid(
        websign_protocol::parse_client_message(raw, Some(1)),
        id("1"),
    );
    assert_names_field_not_value(with(sign_digest(), "seq", json!(1.0)), "seq", "1.0");
}
