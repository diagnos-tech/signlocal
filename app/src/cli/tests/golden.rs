//! The JSON documents and exit codes callers depend on, byte for byte.

use std::process::ExitCode;

use websign_protocol::messages::ChooseResult;
use websign_protocol::types::{AppInfo, Channel, OsName};
use websign_protocol::{AppMessage, ErrorCode, ProtocolRange};
use websign_registration::Outcome;

use crate::cli::options::HashArg;
use crate::cli::output::{error, message_json};
use crate::cli::report::{Kind, Report, Step};
use crate::cli::sign::{SignArgs, prepare};
use crate::cli::version::render;

fn info() -> AppInfo {
    AppInfo {
        version: "1.4.0".to_owned(),
        protocols: ProtocolRange { min: 1, max: 1 },
        os: OsName::Linux,
        arch: "x86_64".to_owned(),
        channel: Channel::Direct,
    }
}

#[test]
fn version_text_and_json() {
    assert_eq!(
        render(&info(), false).unwrap(),
        "websign 1.4.0 (protocol 1)"
    );
    assert_eq!(
        render(&info(), true).unwrap(),
        r#"{"version":"1.4.0","protocols":{"min":1,"max":1},"os":"linux","arch":"x86_64","channel":"direct"}"#
    );
}

#[test]
fn sign_and_choose_print_the_final_message_without_envelope() {
    assert_eq!(
        message_json(&error(ErrorCode::UserCancelled, "the person cancelled")),
        r#"{"type":"error","code":"UserCancelled","message":"the person cancelled"}"#
    );
    let empty = AppMessage::ChooseResult(ChooseResult {
        certificates: vec![],
    });
    assert_eq!(
        message_json(&empty),
        r#"{"type":"choose.result","certificates":[]}"#
    );
}

#[test]
fn exit_codes_follow_the_table() {
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
        (ErrorCode::AppMissing, 15),
    ];
    for (code, exit) in expected {
        assert_eq!(code.exit_code(), exit, "{code:?}");
    }
}

fn sign_args(digest: &str, certificate: Option<&str>) -> SignArgs {
    SignArgs {
        hash: HashArg::Sha256,
        digest: Some(digest.to_owned()),
        digest_file: None,
        algorithms: vec![],
        certificate: certificate.map(str::to_owned),
    }
}

#[test]
fn a_malformed_digest_or_fingerprint_is_invalid_request_before_any_window() {
    let refused = |args: SignArgs| match prepare(&args) {
        Err(message) => match *message {
            AppMessage::Error(e) => e.code,
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    };
    assert_eq!(refused(sign_args("00", None)), ErrorCode::InvalidRequest);
    assert_eq!(
        refused(sign_args(&"ab".repeat(32), Some("xyz"))),
        ErrorCode::InvalidRequest
    );
    let fp = "AB:".repeat(31) + "AB";
    let (begin, digest) = prepare(&sign_args(&"ab".repeat(32), Some(&fp))).unwrap();
    assert_eq!(digest, vec![0xab; 32]);
    assert_eq!(begin.certificate.unwrap().as_str(), "ab".repeat(32));
    assert!(begin.algorithms.is_none() && begin.web.is_none());
}

#[test]
fn registration_report_json_and_exit_code() {
    let mut report = Report {
        command: "install",
        system: false,
        dry_run: true,
        steps: vec![
            Step {
                kind: Kind::Manifest,
                target: "Google Chrome".to_owned(),
                location: "/h/.config/google-chrome/NativeMessagingHosts/dev.websign.host.json"
                    .to_owned(),
                outcome: Outcome::DryRun,
            },
            Step {
                kind: Kind::UrlScheme,
                target: "websign: URLs".to_owned(),
                location: String::new(),
                outcome: Outcome::Skipped("xdg-mime is missing".to_owned()),
            },
        ],
    };
    assert_eq!(
        report.to_json().to_string(),
        r#"{"command":"install","dryRun":true,"ok":true,"scope":"user","steps":[{"kind":"manifest","location":"/h/.config/google-chrome/NativeMessagingHosts/dev.websign.host.json","outcome":"dryRun","target":"Google Chrome"},{"kind":"urlScheme","location":"","outcome":"skipped","reason":"xdg-mime is missing","target":"websign: URLs"}]}"#
    );
    report.steps[0].outcome = Outcome::Failed("denied".to_owned());
    assert!(!report.ok());
    assert!(
        report
            .to_text()
            .starts_with("failed      Google Chrome — /h/"),
        "{}",
        report.to_text()
    );
    assert_eq!(
        ExitCode::from(1),
        ExitCode::from(crate::cli::output::FAILURE)
    );
}

#[test]
fn doctor_json_keys_are_stable() {
    use websign_ui_model::diagnostics::report::{BrowserLine, ReportInput};
    let input = ReportInput {
        app_version: "1.4.0".to_owned(),
        packaging: "deb".to_owned(),
        arch: "x86_64".to_owned(),
        protocol: 1,
        locale: "en".to_owned(),
        os: "Ubuntu 24.04".to_owned(),
        complement: "n/a".to_owned(),
        browsers: vec![BrowserLine {
            name: "chrome".to_owned(),
            host_registered: true,
            ..Default::default()
        }],
        ..Default::default()
    };
    assert_eq!(
        crate::cli::doctor::json::to_json(&input).to_string(),
        r#"{"app":{"arch":"x86_64","locale":"en","packaging":"deb","protocol":1,"version":"1.4.0"},"browsers":[{"extensionVersion":null,"hostRegistered":true,"lastPing":null,"name":"chrome","version":null}],"certificates":{"deduplicated":0,"expiringWithin30Days":0,"hiddenExpired":0,"hiddenLoginOnly":0,"hiddenOther":0,"keys":{},"kinds":{},"usableOs":0,"usablePkcs11":0},"complement":"n/a","devices":[],"os":"Ubuntu 24.04","pkcs11":[],"recentErrors":[]}"#
    );
}
