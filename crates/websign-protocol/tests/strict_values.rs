//! Body parsing, SPEC §5 step 6: wrong types and invalid values, never echoed.

mod common;

use common::*;
use serde_json::{Value, json};

#[test]
fn wrong_types_are_rejected() {
    let cases: Vec<(Value, &str)> = vec![
        (with(sign_begin(), "hash", json!(256)), "hash"),
        (with(sign_begin(), "hash", json!(null)), "hash"),
        (with(sign_begin(), "hash", json!(["SHA-256"])), "hash"),
        (
            with(sign_begin(), "web", json!("https://app.example")),
            "web",
        ),
        (
            with(sign_begin(), "algorithms", json!("ECDSA")),
            "algorithms",
        ),
        (with(sign_begin(), "algorithms", json!([1])), "algorithms"),
        (with(sign_begin(), "certificate", json!(5)), "certificate"),
        (with(sign_digest(), "seq", json!("1")), "seq"),
        (with(sign_digest(), "seq", json!(-1)), "seq"),
        (with(sign_digest(), "seq", json!(1.5)), "seq"),
        (with(sign_digest(), "seq", json!(4_294_967_296u64)), "seq"),
        (with(sign_digest(), "digest", json!(12)), "digest"),
        (
            with(
                json!({ "v": 1, "id": "d", "type": "diagnostics.open" }),
                "tab",
                json!(1),
            ),
            "tab",
        ),
        (
            with(
                json!({ "v": 1, "id": "2", "type": "choose" }),
                "filter",
                json!([]),
            ),
            "filter",
        ),
        (
            with(
                json!({ "v": 1, "id": "2", "type": "choose" }),
                "filter",
                json!({ "algorithms": "ECDSA" }),
            ),
            "algorithms",
        ),
    ];
    for (frame, field) in cases {
        let error = assert_invalid(
            parse_client(&frame, Some(1)),
            id(frame["id"].as_str().unwrap()),
        );
        assert!(
            error.message.contains(field),
            "{field} missing in {:?}",
            error.message
        );
    }
    let mut frame = hello();
    frame["protocols"] = json!({ "min": "1", "max": 1 });
    assert_invalid(parse_client(&frame, None), id("h"));
    let mut frame = hello();
    frame["client"]["name"] = json!(5);
    assert_invalid(parse_client(&frame, None), id("h"));
    let mut frame = need_digest();
    frame["certificate"]["notBefore"] = json!("1741000000");
    assert_invalid(parse_app(&frame, Some(1)), id("3"));
    let mut frame = need_digest();
    frame["certificate"]["notAfter"] = json!(1.5);
    assert_invalid(parse_app(&frame, Some(1)), id("3"));
    assert_invalid(
        parse_app(
            &json!({ "v": 1, "id": "1", "type": "status", "app": app_info(), "remembered": "no" }),
            Some(1),
        ),
        id("1"),
    );
}

#[test]
fn invalid_enum_values_are_rejected_without_echo() {
    let secret = "SECRET-ENUM";
    assert_names_field_not_value(with(sign_begin(), "hash", json!(secret)), "hash", secret);
    assert_names_field_not_value(
        with(sign_begin(), "hash", json!("sha-256")),
        "hash",
        "sha-256",
    );
    assert_names_field_not_value(with(sign_begin(), "hash", json!("SHA-1")), "hash", "SHA-1");
    assert_names_field_not_value(
        with(sign_begin(), "algorithms", json!(["ecdsa"])),
        "algorithms",
        "ecdsa",
    );
    for (field, value) in [("name", "netscape"), ("reason", "boot")] {
        let mut frame = hello();
        frame["browser"][field] = json!(value);
        assert_invalid(parse_client(&frame, None), id("h"));
    }
    let mut frame = need_digest();
    frame["algorithm"] = json!("HMAC");
    assert_invalid(parse_app(&frame, Some(1)), id("3"));
    let mut frame = need_digest();
    frame["certificate"]["key"] = json!({ "type": "DSA", "bits": 1024 });
    assert_invalid(parse_app(&frame, Some(1)), id("3"));
    let mut frame = need_digest();
    frame["certificate"]["key"] = json!({ "type": "EC", "curve": "secp256k1" });
    assert_invalid(parse_app(&frame, Some(1)), id("3"));
    let error = json!({ "v": 1, "id": "3", "type": "error", "code": "Nope", "message": "m" });
    assert_invalid(parse_app(&error, Some(1)), id("3"));
    let error = json!({ "v": 1, "id": "3", "type": "error", "code": "aborted", "message": "m" });
    assert_invalid(parse_app(&error, Some(1)), id("3"));
    assert_invalid(
        parse_client(
            &json!({ "v": 1, "id": "d", "type": "diagnostics.open", "tab": "settings" }),
            Some(1),
        ),
        id("d"),
    );
}

#[test]
fn invalid_base64_is_rejected_without_echo() {
    let secret = "AAAA AAAA";
    let bad = [
        secret, "AAAA\n", " AAAA", "AA-_", "AAA", "AA=", "AAAAA===", "AB==", "AAB=", "====", "A",
        "AA=A",
    ];
    for text in bad {
        let frame = with(sign_digest(), "digest", json!(text));
        let error = assert_invalid(parse_client(&frame, Some(1)), id("3"));
        assert!(error.message.contains("digest"), "{}", error.message);
        assert!(
            !error.message.contains(text.trim()) || text.trim().len() < 2,
            "{}",
            error.message
        );
    }
    for text in ["", "AAAA", "AA==", "AAA="] {
        assert!(
            parse_client(&with(sign_digest(), "digest", json!(text)), Some(1)).is_ok(),
            "{text:?}"
        );
    }
    let mut frame = need_digest();
    frame["certificate"]["der"] = json!("MII");
    assert_invalid(parse_app(&frame, Some(1)), id("3"));
    let mut frame = need_digest();
    frame["certificate"]["chain"] = json!(["MIIG", "not base64!"]);
    assert_invalid(parse_app(&frame, Some(1)), id("3"));
    assert_invalid(
        parse_app(&with(sign_result(), "signature", json!("Qm9")), Some(1)),
        id("3"),
    );
}

#[test]
fn invalid_fingerprints_are_rejected() {
    let bad = [
        FINGERPRINT.to_uppercase(),
        FINGERPRINT[..63].to_string(),
        format!("{FINGERPRINT}0"),
        format!("{}:{}", &FINGERPRINT[..2], &FINGERPRINT[3..]),
        "g".repeat(64),
        String::new(),
    ];
    for text in bad {
        let frame = with(sign_begin(), "certificate", json!(text));
        let error = assert_invalid(parse_client(&frame, Some(1)), id("3"));
        assert!(error.message.contains("certificate"), "{}", error.message);
        let mut certificate = certificate();
        certificate["fingerprint"] = json!(text);
        assert_invalid(
            parse_app(&with(need_digest(), "certificate", certificate), Some(1)),
            id("3"),
        );
    }
}

#[test]
fn body_field_names_are_camel_case_only() {
    let mut frame = sign_begin();
    frame["web"] = json!({ "origin": "https://a", "top_origin": "https://a" });
    assert_invalid(parse_client(&frame, Some(1)), id("3"));
    let mut frame = need_digest();
    frame["certificate"] = {
        let mut certificate = certificate();
        let name = certificate
            .as_object_mut()
            .unwrap()
            .remove("displayName")
            .unwrap();
        certificate["display_name"] = name;
        certificate
    };
    assert_invalid(parse_app(&frame, Some(1)), id("3"));
}

#[test]
fn field_names_are_case_sensitive() {
    assert_invalid(
        parse_client(
            &with(without(sign_begin(), "hash"), "Hash", json!("SHA-256")),
            Some(1),
        ),
        id("3"),
    );
    let frame = json!({ "V": 1, "id": "3", "type": "cancel" });
    assert_invalid(parse_client(&frame, Some(1)), id("3"));
}

#[test]
fn error_details_accept_each_documented_key() {
    let frame = json!({
        "v": 1, "id": "3", "type": "error", "code": "DriverFailure", "message": "m",
        "details": { "native": "0x1" }
    });
    assert!(parse_app(&frame, Some(1)).is_ok());
    let frame = json!({
        "v": 1, "id": "3", "type": "error", "code": "AppOutdated", "message": "m",
        "details": {}
    });
    assert!(parse_app(&frame, Some(1)).is_ok());
}
