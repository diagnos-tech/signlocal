//! Client frames as a page or a desktop program would send them.

use serde_json::{Value, json};
use websign_protocol::base64;

fn frame(id: &str, kind: &str, fields: Value) -> Vec<u8> {
    frame_v(1, id, kind, fields)
}

/// A frame at protocol version `v`.
fn frame_v(v: u32, id: &str, kind: &str, fields: Value) -> Vec<u8> {
    let mut object = json!({ "v": v, "id": id, "type": kind });
    if let (Some(target), Some(extra)) = (object.as_object_mut(), fields.as_object()) {
        for (key, value) in extra {
            target.insert(key.clone(), value.clone());
        }
    }
    serde_json::to_vec(&object).expect("json")
}

/// `web` as the extension attaches it.
pub fn web(origin: &str) -> Value {
    json!({ "origin": origin, "topOrigin": origin })
}

/// `web` for a request from a frame of `origin` inside a page of `top`.
pub fn framed(origin: &str, top: &str) -> Value {
    json!({ "origin": origin, "topOrigin": top })
}

/// A `hello` sent at version `min`, one the client speaks.
pub fn hello_native(id: &str, min: u32, max: u32) -> Vec<u8> {
    frame_v(
        min.max(1),
        id,
        "hello",
        json!({
            "client": { "name": "websign-extension", "version": "1.4.2" },
            "protocols": { "min": min, "max": max },
            "browser": { "name": "chrome", "version": "129.0", "reason": "page" },
        }),
    )
}

pub fn hello_desktop(id: &str, min: u32, max: u32) -> Vec<u8> {
    frame_v(
        min.max(1),
        id,
        "hello",
        json!({
            "client": { "name": "websign-client", "version": "1.0.0" },
            "protocols": { "min": min, "max": max },
        }),
    )
}

/// A `hello` with `browser` while on the desktop transport, or without it on
/// native messaging, is built by hand from these two.
pub fn hello_without_browser(id: &str) -> Vec<u8> {
    hello_desktop(id, 1, 1)
}

pub fn hello_with_extra_field(id: &str) -> Vec<u8> {
    frame(
        id,
        "hello",
        json!({
            "client": { "name": "x", "version": "1" },
            "protocols": { "min": 1, "max": 1 },
            "browser": { "name": "chrome", "version": "129.0", "reason": "page" },
            "extra": true,
        }),
    )
}

pub fn status(id: &str, origin: Option<&str>) -> Vec<u8> {
    frame(id, "status", with_web(origin, json!({})))
}

pub fn choose(id: &str, origin: Option<&str>) -> Vec<u8> {
    frame(id, "choose", with_web(origin, json!({})))
}

pub fn choose_filtered(id: &str, origin: &str, algorithms: &[&str]) -> Vec<u8> {
    let fields = json!({ "filter": { "algorithms": algorithms } });
    frame(id, "choose", with_web(Some(origin), fields))
}

pub fn sign_begin(id: &str, origin: Option<&str>, hash: &str) -> Vec<u8> {
    frame(id, "sign.begin", with_web(origin, json!({ "hash": hash })))
}

pub fn sign_begin_with(id: &str, origin: &str, fields: Value) -> Vec<u8> {
    let mut fields = fields;
    if let Some(map) = fields.as_object_mut() {
        map.entry("hash").or_insert(json!("SHA-256"));
    }
    frame(id, "sign.begin", with_web(Some(origin), fields))
}

pub fn sign_begin_raw_web(id: &str, web: Value, hash: &str) -> Vec<u8> {
    frame(id, "sign.begin", json!({ "web": web, "hash": hash }))
}

pub fn digest(id: &str, seq: u32, bytes: &[u8]) -> Vec<u8> {
    frame(
        id,
        "sign.digest",
        json!({ "seq": seq, "digest": base64::encode(bytes) }),
    )
}

pub fn cancel(id: &str) -> Vec<u8> {
    frame(id, "cancel", json!({}))
}

pub fn diagnostics(id: &str, tab: Option<&str>) -> Vec<u8> {
    match tab {
        Some(tab) => frame(id, "diagnostics.open", json!({ "tab": tab })),
        None => frame(id, "diagnostics.open", json!({})),
    }
}

fn with_web(origin: Option<&str>, mut fields: Value) -> Value {
    if let (Some(origin), Some(map)) = (origin, fields.as_object_mut()) {
        map.insert("web".to_owned(), web(origin));
    }
    fields
}
