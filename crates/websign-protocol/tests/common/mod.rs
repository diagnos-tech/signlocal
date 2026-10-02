//! Shared fixtures: golden JSON taken from `docs/architecture/protocol.md`.
#![allow(dead_code)]

use serde_json::{Value, json};
use websign_protocol::{AppEnvelope, ClientEnvelope, ErrorCode, ParseError, RequestId};

pub const FINGERPRINT: &str = "256adb9a256adb9a256adb9a256adb9a256adb9a256adb9a256adb9a256adb9a";
pub const DIGEST_B64: &str = "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg=";

pub fn certificate() -> Value {
    json!({
        "der": "MIIF",
        "chain": ["MIIG"],
        "fingerprint": FINGERPRINT,
        "displayName": "Ana Beatriz Souza",
        "issuerName": "AC SOLUTI Multipla v5",
        "notBefore": 1741000000,
        "notAfter": 1792000000,
        "key": { "type": "RSA", "bits": 2048 },
        "algorithms": ["RSASSA-PKCS1-v1_5", "RSASSA-PSS"],
        "profile": { "icpBrasil": "A3", "keyStorage": "hardware" }
    })
}

pub fn app_info() -> Value {
    json!({
        "version": "1.4.0",
        "protocols": { "min": 1, "max": 1 },
        "os": "windows",
        "arch": "x86_64",
        "channel": "direct"
    })
}

pub fn web() -> Value {
    json!({ "origin": "https://app.example", "topOrigin": "https://app.example" })
}

pub fn hello() -> Value {
    json!({
        "v": 1, "id": "h", "type": "hello",
        "client": { "name": "websign-extension", "version": "1.4.2" },
        "protocols": { "min": 1, "max": 1 },
        "browser": { "name": "chrome", "version": "129.0", "reason": "page" }
    })
}

pub fn hello_reply() -> Value {
    json!({ "v": 1, "id": "h", "type": "hello", "protocol": 1, "app": app_info() })
}

pub fn sign_begin() -> Value {
    json!({ "v": 1, "id": "3", "type": "sign.begin", "web": web(), "hash": "SHA-256" })
}

pub fn sign_digest() -> Value {
    json!({ "v": 1, "id": "3", "type": "sign.digest", "seq": 1, "digest": DIGEST_B64 })
}

pub fn need_digest() -> Value {
    json!({
        "v": 1, "id": "3", "type": "sign.need_digest", "seq": 1, "hash": "SHA-256",
        "algorithm": "RSASSA-PKCS1-v1_5", "certificate": certificate()
    })
}

pub fn sign_result() -> Value {
    json!({
        "v": 1, "id": "3", "type": "sign.result", "hash": "SHA-256",
        "algorithm": "RSASSA-PKCS1-v1_5", "certificate": certificate(), "signature": "Qm9v"
    })
}

pub fn to_bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("serialize fixture")
}

pub fn from_bytes(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("valid json")
}

pub fn parse_client(value: &Value, negotiated: Option<u32>) -> Result<ClientEnvelope, ParseError> {
    websign_protocol::parse_client_message(&to_bytes(value), negotiated)
}

pub fn parse_app(value: &Value, negotiated: Option<u32>) -> Result<AppEnvelope, ParseError> {
    websign_protocol::parse_app_message(&to_bytes(value), negotiated)
}

/// Returns `value` with the top-level `key` set to `new`.
pub fn with(mut value: Value, key: &str, new: Value) -> Value {
    value[key] = new;
    value
}

/// Returns `value` without the top-level `key`.
pub fn without(mut value: Value, key: &str) -> Value {
    value.as_object_mut().expect("object").remove(key);
    value
}

/// A valid request id, as the `id` an error must carry.
pub fn id(text: &str) -> Option<RequestId> {
    Some(RequestId::new(text).unwrap())
}

pub fn assert_invalid(
    result: Result<impl std::fmt::Debug, ParseError>,
    expected_id: Option<RequestId>,
) -> ParseError {
    let error = result.expect_err("must be rejected");
    assert_eq!(error.code, ErrorCode::InvalidRequest, "{error:?}");
    assert_eq!(error.id, expected_id, "{error:?}");
    error
}

/// Rejected bodies name the field but never echo the value.
pub fn assert_names_field_not_value(frame: Value, field: &str, secret: &str) {
    let error = assert_invalid(
        parse_client(&frame, Some(1)),
        id(frame["id"].as_str().unwrap()),
    );
    assert!(
        error.message.contains(field),
        "{field} missing in {:?}",
        error.message
    );
    assert!(
        !error.message.contains(secret),
        "value echoed in {:?}",
        error.message
    );
}
