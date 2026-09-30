//! SPEC §3.3 and ux §8.7: the exact "Copy diagnostics" text.

use websign_ui_model::diagnostics::report::{
    BrowserLine, CertificateCounts, DeviceLine, ErrorLine, ModuleLine, ReportInput, render,
};

const ATR: &str = "3B:D5:18:FF:81:91:FE:1F:C3:80:73:C8:21:10:0A";

fn s(text: &str) -> String {
    text.to_owned()
}

/// The example of ux §8.7, field by field.
fn example() -> ReportInput {
    ReportInput {
        app_version: s("1.4.0"),
        packaging: s("msix"),
        arch: s("x86_64"),
        protocol: 1,
        locale: s("pt-BR"),
        scale_percent: 125,
        os: s("Windows 11 23H2 (10.0.22631)"),
        render: s("wgpu/dx12"),
        browsers: vec![
            BrowserLine {
                name: s("chrome"),
                version: Some(s("129.0")),
                extension_version: Some(s("1.4.2")),
                host_registered: true,
                last_ping: Some(s("2026-09-29T14:02Z")),
            },
            BrowserLine {
                name: s("edge"),
                version: Some(s("129.0")),
                extension_version: None,
                host_registered: true,
                last_ping: None,
            },
            BrowserLine {
                name: s("firefox"),
                version: Some(s("131.0")),
                extension_version: Some(s("1.4.2")),
                host_registered: true,
                last_ping: Some(s("2026-09-28T20:40Z")),
            },
        ],
        devices: vec![
            DeviceLine::Usb {
                vid_pid: s("0529:0620"),
                hint_id: Some(s("safenet-etoken-5110")),
                certs: 0,
            },
            DeviceLine::Reader {
                name: s("Identiv uTrust 2700 R"),
                atr: Some(s(ATR)),
                certs: 1,
            },
        ],
        modules: vec![
            ModuleLine {
                path: s(r"%ProgramFiles%\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll"),
                result: Ok((1, 1)),
                user_added: false,
            },
            ModuleLine {
                path: s(r"%USERPROFILE%\Downloads\wdpkcs_icp.dll"),
                result: Err(s("file not found")),
                user_added: true,
            },
        ],
        certificates: CertificateCounts {
            usable_os: 3,
            usable_pkcs11: 0,
            deduplicated: 1,
            hidden_expired: 1,
            hidden_login_only: 1,
            hidden_other: 0,
            kinds: vec![(s("icp-brasil-a3"), 1), (s("icp-brasil-a1"), 2)],
            keys: vec![(s("rsa-2048"), 3)],
            expiring_within_30_days: 2,
        },
        complement: s("n/a"),
        recent_errors: vec![ErrorLine {
            at: s("2026-09-29T14:05Z"),
            operation: s("sign"),
            code: s("PinIncorrect"),
            source: s("pkcs11"),
            native: Some(s("CKR_PIN_INCORRECT")),
        }],
    }
}

const GOLDEN: &str = "\
WebeSign diagnostics v1
app: 1.4.0 (msix, x86_64) · protocol 1 · locale pt-BR · scale 125%
os: Windows 11 23H2 (10.0.22631)
render: wgpu/dx12
browsers:
  chrome 129.0 · extension 1.4.2 · host registered · last ping 2026-09-29T14:02Z
  edge 129.0 · extension not seen · host registered
  firefox 131.0 · extension 1.4.2 · host registered · last ping 2026-09-28T20:40Z
devices:
  usb 0529:0620 safenet-etoken-5110 · certs 0
  reader \"Identiv uTrust 2700 R\" · atr 3B:D5:18:FF:81:91:FE:1F:C3:80:73:C8:21:10:0A · certs 1
pkcs11:
  %ProgramFiles%\\OpenSC Project\\OpenSC\\pkcs11\\opensc-pkcs11.dll · loaded · slots 1 · tokens 1
  %USERPROFILE%\\Downloads\\wdpkcs_icp.dll · failed: file not found (user-added)
certificates:
  usable 3 (os 3, pkcs11 0, deduplicated 1) · hidden 2 (expired 1, login-only 1)
  kinds: icp-brasil-a3 1, icp-brasil-a1 2 · keys: rsa-2048 3
  expiring<=30d 2
complement: n/a
recent errors (last 20):
  2026-09-29T14:05Z sign PinIncorrect pkcs11 CKR_PIN_INCORRECT
";

#[test]
fn the_ux_example_is_reproduced_byte_for_byte() {
    assert_eq!(render(&example()), GOLDEN);
}

#[test]
fn the_text_ends_with_exactly_one_newline() {
    let text = render(&example());
    assert!(text.ends_with('\n'));
    assert!(!text.ends_with("\n\n"));
}

#[test]
fn rendering_is_deterministic() {
    assert_eq!(render(&example()), render(&example()));
}

fn empty_sections() -> ReportInput {
    ReportInput {
        browsers: vec![],
        devices: vec![],
        modules: vec![],
        recent_errors: vec![],
        ..example()
    }
}

#[test]
fn empty_sections_print_none() {
    let text = render(&empty_sections());
    let expected = "\
WebeSign diagnostics v1
app: 1.4.0 (msix, x86_64) · protocol 1 · locale pt-BR · scale 125%
os: Windows 11 23H2 (10.0.22631)
render: wgpu/dx12
browsers:
  none
devices:
  none
pkcs11:
  none
certificates:
  usable 3 (os 3, pkcs11 0, deduplicated 1) · hidden 2 (expired 1, login-only 1)
  kinds: icp-brasil-a3 1, icp-brasil-a1 2 · keys: rsa-2048 3
  expiring<=30d 2
complement: n/a
recent errors (last 20):
  none
";
    assert_eq!(text, expected);
}

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

#[test]
fn the_complement_line_is_printed_as_given() {
    let text = render(&ReportInput {
        complement: s("1.2.0 (outdated)"),
        ..example()
    });
    assert!(text.contains("\ncomplement: 1.2.0 (outdated)\n"));
}

#[test]
fn nothing_personal_can_appear_because_the_input_has_no_place_for_it() {
    // The report is built only from the fields above; whatever is in them is
    // printed, and everything else about the machine is simply not in them.
    let text = render(&example());
    for forbidden in [
        "serial",
        "fingerprint",
        "digest",
        "cpf",
        "cnpj",
        "@",
        "https://",
    ] {
        assert!(
            !text.to_lowercase().contains(forbidden),
            "the golden text must not mention {forbidden:?}"
        );
    }
}

#[test]
fn every_line_is_plain_english_ascii_apart_from_the_separator_dot() {
    for line in render(&example()).lines() {
        assert!(
            line.chars().all(|c| c.is_ascii() || c == '·'),
            "unexpected character in {line:?}"
        );
    }
}
