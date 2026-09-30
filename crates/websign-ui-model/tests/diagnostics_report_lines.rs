//! SPEC §3.3 and ux §8.7: each line of the "Copy diagnostics" text.

mod common;

use common::*;
use websign_ui_model::diagnostics::report::{
    BrowserLine, CertificateCounts, DeviceLine, ErrorLine, ModuleLine, ReportInput, render,
};

fn browser_line(input: BrowserLine) -> String {
    let text = render(&ReportInput {
        browsers: vec![input],
        ..empty_sections()
    });
    text.lines().nth(5).expect("first browser line").to_owned()
}

#[test]
fn an_unknown_browser_version_prints_a_question_mark() {
    let line = browser_line(BrowserLine {
        name: s("safari"),
        version: None,
        extension_version: Some(s("1.4.2")),
        host_registered: true,
        last_ping: None,
    });
    assert_eq!(line, "  safari ? · extension 1.4.2 · host registered");
}

#[test]
fn an_unregistered_host_and_a_missing_extension_are_spelled_out() {
    let line = browser_line(BrowserLine {
        name: s("brave"),
        version: Some(s("1.70")),
        extension_version: None,
        host_registered: false,
        last_ping: None,
    });
    assert_eq!(
        line,
        "  brave 1.70 · extension not seen · host not registered"
    );
}

#[test]
fn the_last_ping_is_appended_only_when_known() {
    let line = browser_line(BrowserLine {
        name: s("chrome"),
        version: Some(s("129.0")),
        extension_version: Some(s("1.4.2")),
        host_registered: false,
        last_ping: Some(s("2026-09-29T14:02Z")),
    });
    assert_eq!(
        line,
        "  chrome 129.0 · extension 1.4.2 · host not registered · last ping 2026-09-29T14:02Z"
    );
}

fn device_lines(devices: Vec<DeviceLine>) -> Vec<String> {
    let text = render(&ReportInput {
        devices,
        ..empty_sections()
    });
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| *l == "devices:").unwrap() + 1;
    let end = lines.iter().position(|l| *l == "pkcs11:").unwrap();
    lines[start..end].iter().map(|l| (*l).to_owned()).collect()
}

#[test]
fn a_usb_device_without_a_hint_is_unknown() {
    let lines = device_lines(vec![DeviceLine::Usb {
        vid_pid: s("1234:abcd"),
        hint_id: None,
        certs: 2,
    }]);
    assert_eq!(lines, vec!["  usb 1234:abcd unknown · certs 2"]);
}

#[test]
fn a_reader_without_a_card_has_no_atr() {
    let lines = device_lines(vec![DeviceLine::Reader {
        name: s("ACS ACR39U"),
        atr: None,
        certs: 0,
    }]);
    assert_eq!(lines, vec![r#"  reader "ACS ACR39U" · atr none · certs 0"#]);
}

fn module_lines(modules: Vec<ModuleLine>) -> Vec<String> {
    let text = render(&ReportInput {
        modules,
        ..empty_sections()
    });
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| *l == "pkcs11:").unwrap() + 1;
    let end = lines.iter().position(|l| *l == "certificates:").unwrap();
    lines[start..end].iter().map(|l| (*l).to_owned()).collect()
}

#[test]
fn modules_mark_user_added_ones_in_both_outcomes() {
    let lines = module_lines(vec![
        ModuleLine {
            path: s("~/lib/custom.so"),
            result: Ok((2, 0)),
            user_added: true,
        },
        ModuleLine {
            path: s("/usr/lib/opensc-pkcs11.so"),
            result: Err(s("driver returned CKR_GENERAL_ERROR")),
            user_added: false,
        },
    ]);
    assert_eq!(
        lines,
        vec![
            "  ~/lib/custom.so · loaded · slots 2 · tokens 0 (user-added)",
            "  /usr/lib/opensc-pkcs11.so · failed: driver returned CKR_GENERAL_ERROR",
        ]
    );
}

fn certificate_lines(counts: CertificateCounts) -> Vec<String> {
    let text = render(&ReportInput {
        certificates: counts,
        ..empty_sections()
    });
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| *l == "certificates:").unwrap() + 1;
    let end = lines
        .iter()
        .position(|l| l.starts_with("complement:"))
        .unwrap();
    lines[start..end].iter().map(|l| (*l).to_owned()).collect()
}

#[test]
fn hidden_is_the_sum_and_other_appears_only_when_not_zero() {
    let lines = certificate_lines(CertificateCounts {
        usable_os: 1,
        usable_pkcs11: 2,
        deduplicated: 0,
        hidden_expired: 1,
        hidden_login_only: 4,
        hidden_other: 3,
        kinds: vec![(s("generic"), 3)],
        keys: vec![(s("ecdsa-p256"), 1), (s("rsa-2048"), 2)],
        expiring_within_30_days: 0,
    });
    assert_eq!(
        lines,
        vec![
            "  usable 3 (os 1, pkcs11 2, deduplicated 0) · hidden 8 (expired 1, login-only 4, other 3)",
            "  kinds: generic 3 · keys: ecdsa-p256 1, rsa-2048 2",
            "  expiring<=30d 0",
        ]
    );
}

#[test]
fn recent_errors_keep_their_order_and_omit_a_missing_native_status() {
    let text = render(&ReportInput {
        recent_errors: vec![
            ErrorLine {
                at: s("2026-09-29T13:00Z"),
                operation: s("list"),
                code: s("DriverFailure"),
                source: s("pkcs11"),
                native: Some(s("CKR_DEVICE_ERROR (0x00000030)")),
            },
            ErrorLine {
                at: s("2026-09-29T14:05Z"),
                operation: s("sign"),
                code: s("UserCancelled"),
                source: s("ui"),
                native: None,
            },
        ],
        ..empty_sections()
    });
    assert!(text.ends_with(
        "recent errors (last 20):\n\
         \x20 2026-09-29T13:00Z list DriverFailure pkcs11 CKR_DEVICE_ERROR (0x00000030)\n\
         \x20 2026-09-29T14:05Z sign UserCancelled ui\n"
    ));
}
