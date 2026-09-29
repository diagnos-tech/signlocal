use super::*;

fn parse(text: &str) -> Parsed {
    parse_request(text.as_bytes())
}

fn code_of(parsed: &Parsed) -> ErrorCode {
    parsed.request.as_ref().unwrap_err().code
}

#[test]
fn parses_ping_with_and_without_client_info() {
    let bare = parse(r#"{"v":1,"id":"a","type":"ping"}"#);
    assert_eq!(bare.id.as_deref(), Some("a"));
    assert!(matches!(bare.request, Ok(Request::Ping(None))));

    let full = parse(
        r#"{"v":1,"id":"b","type":"ping","client":{"extension_version":"0.1.0","reason":"startup"}}"#,
    );
    let Ok(Request::Ping(Some(client))) = full.request else {
        panic!("expected ping with client info");
    };
    assert_eq!(client.extension_version.as_deref(), Some("0.1.0"));
    assert_eq!(client.reason.as_deref(), Some("startup"));
}

#[test]
fn parses_list_and_sign() {
    assert!(matches!(
        parse(r#"{"v":1,"id":"c","type":"list"}"#).request,
        Ok(Request::List)
    ));
    let sign = parse(
        r#"{"v":1,"id":"d","type":"sign","fingerprint":"ab","hash":"SHA-256","algorithm":"ECDSA","digest":"AAAA","extra":true}"#,
    );
    let Ok(Request::Sign(fields)) = sign.request else {
        panic!("expected sign");
    };
    assert_eq!(fields.hash, "SHA-256");
    assert_eq!(fields.digest, "AAAA");
}

#[test]
fn rejects_other_versions_with_a_typed_error_that_keeps_the_id() {
    for version in ["2", "0", "1000"] {
        let parsed = parse(&format!(r#"{{"v":{version},"id":"x","type":"ping"}}"#));
        assert_eq!(parsed.id.as_deref(), Some("x"));
        assert_eq!(code_of(&parsed), ErrorCode::UnsupportedVersion);
    }
}

#[test]
fn a_missing_or_non_numeric_version_is_a_bad_request() {
    for text in [
        r#"{"id":"x","type":"ping"}"#,
        r#"{"v":"1","id":"x","type":"ping"}"#,
        r#"{"v":null,"id":"x","type":"ping"}"#,
    ] {
        assert_eq!(code_of(&parse(text)), ErrorCode::BadRequest, "{text}");
    }
}

#[test]
fn a_fractional_version_is_unsupported() {
    let parsed = parse(r#"{"v":1.5,"id":"x","type":"ping"}"#);
    assert_eq!(code_of(&parsed), ErrorCode::UnsupportedVersion);
}

#[test]
fn invalid_json_and_non_objects_are_bad_requests_without_an_id() {
    for text in ["", "not json", "{", "[]", "42", "\"ping\"", "null"] {
        let parsed = parse(text);
        assert_eq!(parsed.id, None);
        assert_eq!(code_of(&parsed), ErrorCode::BadRequest, "{text:?}");
    }
    let invalid_utf8 = parse_request(&[b'{', 0xff, b'}']);
    assert_eq!(code_of(&invalid_utf8), ErrorCode::BadRequest);
}

#[test]
fn oversized_or_non_string_ids_are_refused() {
    let long = "x".repeat(MAX_ID_LEN + 1);
    let parsed = parse(&format!(r#"{{"v":1,"id":"{long}","type":"ping"}}"#));
    assert_eq!(parsed.id, None);
    assert_eq!(code_of(&parsed), ErrorCode::BadRequest);
    assert_eq!(
        code_of(&parse(r#"{"v":1,"id":7,"type":"ping"}"#)),
        ErrorCode::BadRequest
    );
}

#[test]
fn unknown_and_missing_types_are_typed() {
    assert_eq!(
        code_of(&parse(r#"{"v":1,"id":"x","type":"format-disk"}"#)),
        ErrorCode::UnknownType
    );
    assert_eq!(
        code_of(&parse(r#"{"v":1,"id":"x"}"#)),
        ErrorCode::BadRequest
    );
}

#[test]
fn sign_requires_all_four_fields() {
    for text in [
        r#"{"v":1,"id":"x","type":"sign"}"#,
        r#"{"v":1,"id":"x","type":"sign","fingerprint":"a","hash":"SHA-256","algorithm":"ECDSA"}"#,
        r#"{"v":1,"id":"x","type":"sign","fingerprint":1,"hash":"SHA-256","algorithm":"ECDSA","digest":"AA=="}"#,
    ] {
        assert_eq!(code_of(&parse(text)), ErrorCode::BadRequest, "{text}");
    }
}

#[test]
fn replies_carry_the_envelope() {
    let ok = success(Some("7"), "pong", json!({ "os": "linux" }));
    assert_eq!(ok["v"], 1);
    assert_eq!(ok["id"], "7");
    assert_eq!(ok["ok"], true);
    assert_eq!(ok["type"], "pong");
    assert_eq!(ok["os"], "linux");

    let err = failure(
        None,
        &ProtocolError::new(ErrorCode::NotFound, "no such key"),
    );
    assert_eq!(err["ok"], false);
    assert_eq!(err["id"], Value::Null);
    assert_eq!(err["error"]["code"], "not_found");
    assert_eq!(err["error"]["message"], "no such key");
}

#[test]
fn version_errors_list_the_supported_versions() {
    let err = failure(
        Some("1"),
        &ProtocolError::new(ErrorCode::UnsupportedVersion, "nope"),
    );
    assert_eq!(err["error"]["supported"], json!([1]));
}

#[test]
fn error_codes_are_stable() {
    let all = [
        (ErrorCode::BadRequest, "bad_request"),
        (ErrorCode::UnsupportedVersion, "unsupported_version"),
        (ErrorCode::UnknownType, "unknown_type"),
        (ErrorCode::DigestLength, "digest_length"),
        (ErrorCode::NotFound, "not_found"),
        (ErrorCode::Cancelled, "cancelled"),
        (ErrorCode::WrongPin, "wrong_pin"),
        (ErrorCode::PinRequired, "pin_required"),
        (ErrorCode::PinLocked, "pin_locked"),
        (ErrorCode::Unsupported, "unsupported"),
        (ErrorCode::Internal, "internal"),
    ];
    for (code, name) in all {
        assert_eq!(code.as_str(), name);
    }
}
