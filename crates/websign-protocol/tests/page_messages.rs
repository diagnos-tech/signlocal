//! Page <-> extension messages (protocol.md §9, SPEC §9).

mod common;

use common::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use websign_protocol::page::{
    EXTENSION_SOURCE, ExtensionInfo, ExtensionToPage, PAGE_SOURCE, PageReply, PageRequest,
    PageStatus, PageToExtension,
};

fn golden<T: Serialize + DeserializeOwned + std::fmt::Debug>(json: Value) -> T {
    let parsed: T = serde_json::from_value(json.clone()).unwrap_or_else(|e| panic!("{json}: {e}"));
    assert_eq!(serde_json::to_value(&parsed).unwrap(), json);
    parsed
}

fn rejected<T: DeserializeOwned + std::fmt::Debug>(json: Value) {
    assert!(serde_json::from_value::<T>(json.clone()).is_err(), "{json}");
}

#[test]
fn source_constants() {
    assert_eq!(PAGE_SOURCE, "websign-page");
    assert_eq!(EXTENSION_SOURCE, "websign-extension");
}

#[test]
fn page_to_extension_golden() {
    let parsed: PageToExtension = golden(json!({ "source": "websign-page", "kind": "discover" }));
    assert!(matches!(parsed, PageToExtension::Discover { .. }));
    let parsed: PageToExtension = golden(json!({
        "source": "websign-page", "kind": "request", "id": "p1",
        "message": { "type": "sign.begin", "hash": "SHA-256" }
    }));
    assert!(matches!(parsed, PageToExtension::Request { .. }));
}

#[test]
fn extension_to_page_golden() {
    let parsed: ExtensionToPage = golden(json!({
        "source": "websign-extension", "kind": "announce",
        "extension": { "version": "1.4.2", "browser": "chrome" },
        "protocols": { "min": 1, "max": 1 }
    }));
    assert!(matches!(parsed, ExtensionToPage::Announce { .. }));
    let parsed: ExtensionToPage = golden(json!({
        "source": "websign-extension", "kind": "message", "id": "p1",
        "message": need_digest_reply()
    }));
    assert!(matches!(parsed, ExtensionToPage::Message { .. }));
}

/// A `sign.need_digest` as the page sees it: no envelope `v` or `id`.
fn need_digest_reply() -> Value {
    let mut message = need_digest();
    message.as_object_mut().unwrap().remove("v");
    message.as_object_mut().unwrap().remove("id");
    message
}

#[test]
fn every_page_request_golden() {
    let cases = [
        json!({ "type": "status" }),
        json!({ "type": "choose" }),
        json!({ "type": "choose", "filter": { "algorithms": ["ECDSA"] } }),
        json!({ "type": "sign.begin", "hash": "SHA-384" }),
        json!({ "type": "sign.begin", "hash": "SHA-512", "algorithms": ["RSASSA-PSS"], "certificate": FINGERPRINT }),
        json!({ "type": "sign.digest", "seq": 2, "digest": DIGEST_B64 }),
        json!({ "type": "cancel" }),
    ];
    for case in cases {
        let _: PageRequest = golden(case);
    }
}

#[test]
fn every_page_reply_golden() {
    let mut result = sign_result();
    result.as_object_mut().unwrap().remove("v");
    result.as_object_mut().unwrap().remove("id");
    let cases = [
        json!({
            "type": "status",
            "extension": { "version": "1.4.2", "browser": "firefox" },
            "app": app_info(), "appOutdated": false, "remembered": true
        }),
        json!({
            "type": "status",
            "extension": { "version": "1.4.2", "browser": "edge" },
            "appOutdated": true, "remembered": false
        }),
        json!({ "type": "choose.result", "certificates": [certificate()] }),
        need_digest_reply(),
        result,
        json!({ "type": "error", "code": "ExtensionOutdated", "message": "update the extension" }),
    ];
    for case in cases {
        let _: PageReply = golden(case);
    }
}

#[test]
fn page_requests_are_a_subset_of_the_catalog() {
    for ty in [
        "hello",
        "diagnostics.open",
        "done",
        "sign.result",
        "sign.need_digest",
        "choose.result",
        "error",
        "nope",
    ] {
        rejected::<PageRequest>(json!({ "type": ty }));
    }
    rejected::<PageRequest>(json!({ "type": "status", "web": web() }));
    rejected::<PageRequest>(json!({ "type": "choose", "web": web() }));
    rejected::<PageRequest>(json!({ "type": "sign.begin", "hash": "SHA-256", "web": web() }));
}

#[test]
fn page_replies_are_a_subset_of_the_catalog() {
    for ty in [
        "hello",
        "done",
        "sign.begin",
        "sign.digest",
        "cancel",
        "diagnostics.open",
        "nope",
    ] {
        rejected::<PageReply>(json!({ "type": ty }));
    }
}

#[test]
fn page_requests_are_strict() {
    rejected::<PageRequest>(json!({ "type": "sign.begin" }));
    rejected::<PageRequest>(json!({ "type": "sign.begin", "hash": "MD5" }));
    rejected::<PageRequest>(json!({ "type": "sign.begin", "hash": "SHA-256", "extra": 1 }));
    rejected::<PageRequest>(
        json!({ "type": "sign.begin", "hash": "SHA-256", "certificate": "ABC" }),
    );
    rejected::<PageRequest>(json!({ "type": "sign.digest", "seq": 1 }));
    rejected::<PageRequest>(json!({ "type": "sign.digest", "seq": 1, "digest": "not base64!" }));
    rejected::<PageRequest>(json!({ "type": "status", "extra": 1 }));
    rejected::<PageRequest>(json!({ "type": "cancel", "extra": 1 }));
    rejected::<PageRequest>(json!({ "type": "choose", "filter": { "extra": 1 } }));
    rejected::<PageRequest>(json!({}));
}

#[test]
fn page_status_is_strict() {
    let status = json!({
        "extension": { "version": "1.4.2", "browser": "chrome" }, "appOutdated": false, "remembered": false
    });
    assert!(serde_json::from_value::<PageStatus>(status.clone()).is_ok());
    for field in ["extension", "appOutdated", "remembered"] {
        rejected::<PageStatus>(without(status.clone(), field));
    }
    rejected::<PageStatus>(with(status.clone(), "extra", json!(1)));
    rejected::<PageStatus>(with(status.clone(), "app_outdated", json!(true)));
    rejected::<PageStatus>(with(status, "remembered", json!("yes")));
}

#[test]
fn extension_info_is_strict() {
    let info: ExtensionInfo = golden(json!({ "version": "1.4.2", "browser": "safari" }));
    assert_eq!(info.version, "1.4.2");
    rejected::<ExtensionInfo>(json!({ "version": "1.4.2" }));
    rejected::<ExtensionInfo>(json!({ "version": "1.4.2", "browser": "netscape" }));
    rejected::<ExtensionInfo>(json!({ "version": "1.4.2", "browser": "chrome", "extra": 1 }));
}

#[test]
fn envelope_kinds_are_strict() {
    rejected::<PageToExtension>(json!({ "source": "websign-page", "kind": "nope" }));
    rejected::<PageToExtension>(json!({ "source": "websign-page" }));
    rejected::<PageToExtension>(json!({ "kind": "discover" }));
    rejected::<PageToExtension>(
        json!({ "source": "websign-page", "kind": "request", "message": { "type": "status" } }),
    );
    rejected::<PageToExtension>(
        json!({ "source": "websign-page", "kind": "request", "id": "bad id", "message": { "type": "status" } }),
    );
    rejected::<PageToExtension>(json!({ "source": "websign-page", "kind": "request", "id": "p1" }));
    rejected::<PageToExtension>(
        json!({ "source": "websign-page", "kind": "discover", "extra": 1 }),
    );
    rejected::<ExtensionToPage>(
        json!({ "source": "websign-extension", "kind": "announce", "protocols": { "min": 1, "max": 1 } }),
    );
    rejected::<ExtensionToPage>(
        json!({ "source": "websign-extension", "kind": "message", "message": { "type": "error", "code": "Busy", "message": "m" } }),
    );
    rejected::<ExtensionToPage>(json!({ "source": "websign-extension", "kind": "discover" }));
}

#[test]
fn a_foreign_source_still_parses() {
    // SPEC §9: filtering on `source` is the receiver's job, not the parser's.
    let parsed: PageToExtension =
        serde_json::from_value(json!({ "source": "someone-else", "kind": "discover" })).unwrap();
    match parsed {
        PageToExtension::Discover { source } => assert_eq!(source, "someone-else"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn page_request_ids_follow_the_id_rules() {
    let long = "a".repeat(64);
    let ok = json!({ "source": "websign-page", "kind": "request", "id": long, "message": { "type": "cancel" } });
    assert!(serde_json::from_value::<PageToExtension>(ok).is_ok());
    let too_long = json!({ "source": "websign-page", "kind": "request", "id": "a".repeat(65), "message": { "type": "cancel" } });
    rejected::<PageToExtension>(too_long);
}
