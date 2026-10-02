//! Exact JSON of every message: the examples of `protocol.md` as golden vectors.

mod common;

use common::*;
use serde_json::{Value, json};
use websign_protocol::messages::{AppMessage, ClientMessage};
use websign_protocol::{AppEnvelope, ClientEnvelope, ErrorCode, WireError, to_json};

/// Parses `golden` and checks that serializing gives the same JSON back.
fn client_golden(golden: Value, negotiated: Option<u32>) {
    let envelope = parse_client(&golden, negotiated).expect("golden parses");
    assert_eq!(from_bytes(&to_json(&envelope)), golden);
}

fn app_golden(golden: Value, negotiated: Option<u32>) {
    let envelope = parse_app(&golden, negotiated).expect("golden parses");
    assert_eq!(from_bytes(&to_json(&envelope)), golden);
}

#[test]
fn hello_golden() {
    client_golden(hello(), None);
    client_golden(without(hello(), "browser"), None);
}

#[test]
fn hello_reply_golden() {
    app_golden(hello_reply(), None);
}

#[test]
fn status_golden() {
    client_golden(
        json!({ "v": 1, "id": "1", "type": "status", "web": web() }),
        Some(1),
    );
    client_golden(json!({ "v": 1, "id": "1", "type": "status" }), Some(1));
    app_golden(
        json!({ "v": 1, "id": "1", "type": "status", "app": app_info(), "remembered": false }),
        Some(1),
    );
}

#[test]
fn choose_golden() {
    client_golden(
        json!({
            "v": 1, "id": "2", "type": "choose", "web": web(),
            "filter": { "algorithms": ["ECDSA", "RSASSA-PKCS1-v1_5"] }
        }),
        Some(1),
    );
    client_golden(json!({ "v": 1, "id": "2", "type": "choose" }), Some(1));
    client_golden(
        json!({ "v": 1, "id": "2", "type": "choose", "filter": {} }),
        Some(1),
    );
    app_golden(
        json!({ "v": 1, "id": "2", "type": "choose.result", "certificates": [certificate()] }),
        Some(1),
    );
}

#[test]
fn sign_flow_golden() {
    client_golden(sign_begin(), Some(1));
    client_golden(
        json!({
            "v": 1, "id": "3", "type": "sign.begin", "web": web(), "hash": "SHA-512",
            "algorithms": ["RSASSA-PSS", "ECDSA"], "certificate": FINGERPRINT
        }),
        Some(1),
    );
    app_golden(need_digest(), Some(1));
    client_golden(sign_digest(), Some(1));
    app_golden(sign_result(), Some(1));
}

#[test]
fn control_messages_golden() {
    client_golden(json!({ "v": 1, "id": "3", "type": "cancel" }), Some(1));
    client_golden(
        json!({ "v": 1, "id": "d", "type": "diagnostics.open" }),
        Some(1),
    );
    client_golden(
        json!({ "v": 1, "id": "d", "type": "diagnostics.open", "tab": "devices" }),
        Some(1),
    );
    app_golden(json!({ "v": 1, "id": "d", "type": "done" }), Some(1));
}

#[test]
fn error_golden() {
    app_golden(
        json!({
            "v": 1, "id": "3", "type": "error", "code": "Aborted",
            "message": "the caller cancelled the request"
        }),
        Some(1),
    );
    app_golden(
        json!({
            "v": 1, "id": "3", "type": "error", "code": "AppOutdated", "message": "old",
            "details": { "installed": "1.0.0", "required": "1.4.0", "native": "0x80090016" }
        }),
        Some(1),
    );
}

#[test]
fn certificate_variants_golden() {
    let mut certificate = certificate();
    certificate["key"] = json!({ "type": "EC", "curve": "brainpoolP256r1" });
    certificate["profile"] = json!({
        "eidas": { "qualified": true, "qscd": false, "types": ["esign", "eseal", "web"] },
        "keyStorage": "software"
    });
    certificate["chain"] = json!([]);
    app_golden(
        json!({ "v": 1, "id": "2", "type": "choose.result", "certificates": [certificate] }),
        Some(1),
    );
}

#[test]
fn every_enum_value_has_its_documented_spelling() {
    for hash in ["SHA-256", "SHA-384", "SHA-512"] {
        client_golden(with(sign_begin(), "hash", json!(hash)), Some(1));
    }
    for alg in ["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"] {
        app_golden(with(need_digest(), "algorithm", json!(alg)), Some(1));
    }
    let curves = [
        "P-256",
        "P-384",
        "P-521",
        "brainpoolP256r1",
        "brainpoolP384r1",
        "brainpoolP512r1",
    ];
    for curve in curves {
        let mut certificate = certificate();
        certificate["key"] = json!({ "type": "EC", "curve": curve });
        app_golden(with(need_digest(), "certificate", certificate), Some(1));
    }
    let browsers = [
        "chrome", "chromium", "edge", "brave", "opera", "vivaldi", "firefox", "safari", "other",
    ];
    for browser in browsers {
        let mut golden = hello();
        golden["browser"]["name"] = json!(browser);
        client_golden(golden, None);
    }
    for reason in ["startup", "installed", "page", "popup"] {
        let mut golden = hello();
        golden["browser"]["reason"] = json!(reason);
        client_golden(golden, None);
    }
    for (os, channel) in [
        ("windows", "direct"),
        ("macos", "store"),
        ("linux", "direct"),
    ] {
        let mut info = app_info();
        info["os"] = json!(os);
        info["channel"] = json!(channel);
        app_golden(
            json!({ "v": 1, "id": "1", "type": "status", "app": info, "remembered": true }),
            Some(1),
        );
    }
    for tab in ["browsers", "devices", "certificates", "help"] {
        client_golden(
            json!({ "v": 1, "id": "d", "type": "diagnostics.open", "tab": tab }),
            Some(1),
        );
    }
    for storage in ["hardware", "software", "unknown"] {
        let mut certificate = certificate();
        certificate["profile"]["keyStorage"] = json!(storage);
        app_golden(with(need_digest(), "certificate", certificate), Some(1));
    }
}

#[test]
fn serialization_is_compact() {
    let envelope = parse_client(&sign_begin(), Some(1)).unwrap();
    assert_eq!(to_json(&envelope).len(), to_bytes(&sign_begin()).len());
    let envelope = parse_app(&need_digest(), Some(1)).unwrap();
    assert_eq!(to_json(&envelope).len(), to_bytes(&need_digest()).len());
}

#[test]
fn optional_fields_are_omitted_never_null() {
    let envelope = parse_client(&json!({ "v": 1, "id": "1", "type": "status" }), Some(1)).unwrap();
    assert!(
        !String::from_utf8(to_json(&envelope))
            .unwrap()
            .contains("null")
    );
}

#[test]
fn envelope_keys_are_flat() {
    let value = from_bytes(&to_json(&parse_client(&sign_begin(), Some(1)).unwrap()));
    let mut keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, ["hash", "id", "type", "v", "web"]);
}

#[test]
fn typed_values_serialize_to_the_documented_json() {
    let typed = ClientEnvelope {
        v: 1,
        id: "7".to_string().try_into().unwrap(),
        message: ClientMessage::Cancel(Default::default()),
    };
    assert_eq!(
        from_bytes(&to_json(&typed)),
        json!({ "v": 1, "id": "7", "type": "cancel" })
    );
    let typed = AppEnvelope {
        v: 1,
        id: "7".to_string().try_into().unwrap(),
        message: AppMessage::Error(WireError {
            code: ErrorCode::Busy,
            message: "queue full".into(),
            details: None,
        }),
    };
    assert_eq!(
        from_bytes(&to_json(&typed)),
        json!({ "v": 1, "id": "7", "type": "error", "code": "Busy", "message": "queue full" })
    );
}

#[test]
fn message_helpers_follow_the_catalog() {
    let cases = [
        (hello(), "hello", false),
        (
            json!({ "v": 1, "id": "1", "type": "status" }),
            "status",
            false,
        ),
        (sign_begin(), "sign.begin", false),
        (sign_digest(), "sign.digest", true),
        (
            json!({ "v": 1, "id": "3", "type": "cancel" }),
            "cancel",
            true,
        ),
        (
            json!({ "v": 1, "id": "3", "type": "diagnostics.open" }),
            "diagnostics.open",
            false,
        ),
    ];
    for (golden, kind, continuation) in cases {
        let envelope = parse_client(&golden, if kind == "hello" { None } else { Some(1) }).unwrap();
        assert_eq!(envelope.message.kind(), kind);
        assert_eq!(envelope.message.is_continuation(), continuation);
    }
    assert!(
        parse_app(&sign_result(), Some(1))
            .unwrap()
            .message
            .is_final()
    );
    assert!(
        !parse_app(&need_digest(), Some(1))
            .unwrap()
            .message
            .is_final()
    );
    assert!(
        parse_app(&json!({ "v": 1, "id": "d", "type": "done" }), Some(1))
            .unwrap()
            .message
            .is_final()
    );
}
