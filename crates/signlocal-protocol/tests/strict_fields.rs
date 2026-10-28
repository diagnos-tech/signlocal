//! Body parsing, SPEC §5 step 6: unknown, missing and optional fields (D12).

mod common;

use common::*;
use serde_json::json;

#[test]
fn unknown_fields_are_rejected_at_every_level() {
    assert_names_field_not_value(
        with(sign_begin(), "extra", json!("SECRET-VALUE")),
        "extra",
        "SECRET-VALUE",
    );
    assert_names_field_not_value(
        with(sign_digest(), "extra", json!(1)),
        "extra",
        "SECRET-VALUE",
    );
    assert_names_field_not_value(
        with(
            json!({ "v": 1, "id": "3", "type": "cancel" }),
            "extra",
            json!(1),
        ),
        "extra",
        "SECRET-VALUE",
    );
    assert_names_field_not_value(
        with(
            json!({ "v": 1, "id": "1", "type": "status" }),
            "app",
            json!({}),
        ),
        "app",
        "SECRET-VALUE",
    );

    let mut nested = sign_begin();
    nested["web"]["sneaky"] = json!("SECRET-VALUE");
    assert_names_field_not_value(nested, "sneaky", "SECRET-VALUE");

    let mut nested = sign_begin();
    nested["web"]["top_origin"] = json!("https://x");
    assert_invalid(parse_client(&nested, Some(1)), id("3"));

    let mut nested = hello();
    nested["client"]["extra"] = json!(1);
    assert_invalid(parse_client(&nested, None), id("h"));
    let mut nested = hello();
    nested["browser"]["extra"] = json!(1);
    assert_invalid(parse_client(&nested, None), id("h"));
    let mut nested = hello();
    nested["protocols"]["extra"] = json!(1);
    assert_invalid(parse_client(&nested, None), id("h"));
}

#[test]
fn unknown_fields_are_rejected_in_app_messages_too() {
    assert_invalid(
        parse_app(&with(need_digest(), "extra", json!(1)), Some(1)),
        id("3"),
    );
    let mut certificate = certificate();
    certificate["extra"] = json!(1);
    assert_invalid(
        parse_app(&with(need_digest(), "certificate", certificate), Some(1)),
        id("3"),
    );
    let mut certificate = self::certificate();
    certificate["key"]["extra"] = json!(1);
    assert_invalid(
        parse_app(&with(need_digest(), "certificate", certificate), Some(1)),
        id("3"),
    );
    let mut certificate = self::certificate();
    certificate["profile"]["extra"] = json!(1);
    assert_invalid(
        parse_app(&with(need_digest(), "certificate", certificate), Some(1)),
        id("3"),
    );
    let mut reply = hello_reply();
    reply["app"]["extra"] = json!(1);
    assert_invalid(parse_app(&reply, None), id("h"));
    let error = json!({ "v": 1, "id": "3", "type": "error", "code": "Busy", "message": "m", "details": { "extra": 1 } });
    assert_invalid(parse_app(&error, Some(1)), id("3"));
}

#[test]
fn missing_required_fields_are_rejected() {
    assert_names_field_not_value(without(sign_begin(), "hash"), "hash", "SECRET-VALUE");
    for field in ["seq", "digest"] {
        assert_names_field_not_value(without(sign_digest(), field), field, "SECRET-VALUE");
    }
    for field in ["client", "protocols"] {
        assert_invalid(parse_client(&without(hello(), field), None), id("h"));
    }
    for field in ["origin", "topOrigin"] {
        let mut frame = sign_begin();
        frame["web"].as_object_mut().unwrap().remove(field);
        assert_invalid(parse_client(&frame, Some(1)), id("3"));
    }
    for field in ["seq", "certificate", "hash", "algorithm"] {
        assert_invalid(parse_app(&without(need_digest(), field), Some(1)), id("3"));
    }
    for field in ["certificate", "hash", "algorithm", "signature"] {
        assert_invalid(parse_app(&without(sign_result(), field), Some(1)), id("3"));
    }
    for field in ["app", "protocol"] {
        assert_invalid(parse_app(&without(hello_reply(), field), None), id("h"));
    }
    for field in ["app", "remembered"] {
        let frame = without(
            json!({ "v": 1, "id": "1", "type": "status", "app": app_info(), "remembered": true }),
            field,
        );
        assert_invalid(parse_app(&frame, Some(1)), id("1"));
    }
    assert_invalid(
        parse_app(
            &json!({ "v": 1, "id": "2", "type": "choose.result" }),
            Some(1),
        ),
        id("2"),
    );
    assert_invalid(
        parse_app(
            &json!({ "v": 1, "id": "3", "type": "error", "code": "Busy" }),
            Some(1),
        ),
        id("3"),
    );
    assert_invalid(
        parse_app(
            &json!({ "v": 1, "id": "3", "type": "error", "message": "m" }),
            Some(1),
        ),
        id("3"),
    );
    for field in [
        "der",
        "chain",
        "fingerprint",
        "displayName",
        "issuerName",
        "notBefore",
        "notAfter",
        "key",
        "algorithms",
        "profile",
    ] {
        let mut certificate = certificate();
        certificate.as_object_mut().unwrap().remove(field);
        assert_invalid(
            parse_app(&with(need_digest(), "certificate", certificate), Some(1)),
            id("3"),
        );
    }
    let mut certificate = certificate();
    certificate["profile"]
        .as_object_mut()
        .unwrap()
        .remove("keyStorage");
    assert_invalid(
        parse_app(&with(need_digest(), "certificate", certificate), Some(1)),
        id("3"),
    );
}

#[test]
fn optional_fields_may_be_absent() {
    let parsed = parse_client(
        &json!({ "v": 1, "id": "3", "type": "sign.begin", "hash": "SHA-384" }),
        Some(1),
    );
    assert!(parsed.is_ok());
    let mut certificate = certificate();
    certificate["profile"] = json!({ "keyStorage": "unknown" });
    assert!(parse_app(&with(need_digest(), "certificate", certificate), Some(1)).is_ok());
}
