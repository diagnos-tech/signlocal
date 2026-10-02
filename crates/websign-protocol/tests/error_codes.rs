//! Error codes (protocol.md §7), wire error shape and desktop exit codes.

use std::collections::HashSet;

use serde_json::json;
use websign_protocol::{ErrorCode, ErrorDetails, WireError};

/// The stable names of `protocol.md` §7, in table order.
const NAMES: [&str; 20] = [
    "ExtensionMissing",
    "AppMissing",
    "AppOutdated",
    "ExtensionOutdated",
    "ClientOutdated",
    "InsecureOrigin",
    "Aborted",
    "UserCancelled",
    "Timeout",
    "NoCertificates",
    "CertificateUnavailable",
    "CertificateNotValid",
    "InvalidRequest",
    "UnsupportedAlgorithm",
    "PinIncorrect",
    "PinLocked",
    "TokenRemoved",
    "DriverFailure",
    "Busy",
    "Internal",
];

#[test]
fn all_lists_the_documented_codes_in_order() {
    let names: Vec<_> = ErrorCode::ALL.iter().map(|code| code.as_str()).collect();
    assert_eq!(names, NAMES);
}

#[test]
fn codes_serialize_to_their_stable_string() {
    for code in ErrorCode::ALL {
        assert_eq!(serde_json::to_value(code).unwrap(), json!(code.as_str()));
        assert_eq!(
            serde_json::from_value::<ErrorCode>(json!(code.as_str())).unwrap(),
            code
        );
    }
}

#[test]
fn codes_are_distinct() {
    let unique: HashSet<_> = ErrorCode::ALL.into_iter().collect();
    assert_eq!(unique.len(), NAMES.len());
}

#[test]
fn unknown_or_miscased_codes_are_refused() {
    for bad in [
        "",
        "Nope",
        "aborted",
        "ABORTED",
        "Aborted ",
        "user_cancelled",
    ] {
        assert!(
            serde_json::from_value::<ErrorCode>(json!(bad)).is_err(),
            "{bad:?}"
        );
    }
    assert!(serde_json::from_value::<ErrorCode>(json!(3)).is_err());
}

#[test]
fn exit_codes_match_the_desktop_api_table() {
    let expected = [
        (ErrorCode::Internal, 1),
        (ErrorCode::UserCancelled, 3),
        (ErrorCode::Aborted, 3),
        (ErrorCode::Timeout, 4),
        (ErrorCode::NoCertificates, 5),
        (ErrorCode::CertificateUnavailable, 6),
        (ErrorCode::CertificateNotValid, 7),
        (ErrorCode::UnsupportedAlgorithm, 8),
        (ErrorCode::PinLocked, 9),
        (ErrorCode::PinIncorrect, 10),
        (ErrorCode::TokenRemoved, 11),
        (ErrorCode::DriverFailure, 12),
        (ErrorCode::Busy, 13),
        (ErrorCode::InvalidRequest, 14),
        (ErrorCode::InsecureOrigin, 14),
        (ErrorCode::AppOutdated, 15),
        (ErrorCode::ClientOutdated, 15),
        (ErrorCode::ExtensionOutdated, 15),
        (ErrorCode::AppMissing, 15),
        (ErrorCode::ExtensionMissing, 15),
    ];
    assert_eq!(expected.len(), ErrorCode::ALL.len());
    for (code, exit) in expected {
        assert_eq!(code.exit_code(), exit, "{code:?}");
    }
}

#[test]
fn exit_codes_avoid_success_and_usage_error() {
    for code in ErrorCode::ALL {
        assert!(!matches!(code.exit_code(), 0 | 2), "{code:?}");
    }
}

#[test]
fn wire_error_shape() {
    let error = WireError {
        code: ErrorCode::Timeout,
        message: "too slow".into(),
        details: None,
    };
    assert_eq!(
        serde_json::to_value(&error).unwrap(),
        json!({ "code": "Timeout", "message": "too slow" })
    );

    let error = WireError {
        code: ErrorCode::DriverFailure,
        message: "driver".into(),
        details: Some(ErrorDetails {
            installed: None,
            required: None,
            native: Some("0x8010006B".into()),
        }),
    };
    assert_eq!(
        serde_json::to_value(&error).unwrap(),
        json!({ "code": "DriverFailure", "message": "driver", "details": { "native": "0x8010006B" } })
    );

    let error = WireError {
        code: ErrorCode::AppOutdated,
        message: "old".into(),
        details: Some(ErrorDetails {
            installed: Some("1.0.0".into()),
            required: Some("1.4.0".into()),
            native: None,
        }),
    };
    assert_eq!(
        serde_json::to_value(&error).unwrap(),
        json!({ "code": "AppOutdated", "message": "old", "details": { "installed": "1.0.0", "required": "1.4.0" } })
    );
}

#[test]
fn empty_details_serialize_to_an_empty_object() {
    let error = WireError {
        code: ErrorCode::Busy,
        message: "m".into(),
        details: Some(ErrorDetails::default()),
    };
    assert_eq!(serde_json::to_value(&error).unwrap()["details"], json!({}));
}

#[test]
fn wire_error_parsing_is_strict() {
    for bad in [
        json!({ "code": "Busy" }),
        json!({ "message": "m" }),
        json!({ "code": "Busy", "message": "m", "extra": 1 }),
        json!({ "code": "Busy", "message": 1 }),
        json!({ "code": "Busy", "message": "m", "details": { "other": "x" } }),
        json!({ "code": "Busy", "message": "m", "details": "x" }),
        json!({ "code": "Busy", "message": "m", "details": { "native": 12 } }),
    ] {
        assert!(
            serde_json::from_value::<WireError>(bad.clone()).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn parse_errors_use_invalid_request_and_display() {
    let error = websign_protocol::parse_client_message(b"nope", None).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert!(error.id.is_none());
    assert!(!error.to_string().is_empty());
    assert!(!error.message.is_empty());
}
