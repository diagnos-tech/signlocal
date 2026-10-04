use super::*;

fn example() -> ReportInput {
    ReportInput {
        app_version: "1.4.0".into(),
        packaging: "msix".into(),
        arch: "x86_64".into(),
        protocol: 1,
        locale: "pt-BR".into(),
        scale_percent: 125,
        os: "Windows 11 23H2 (10.0.22631)".into(),
        render: "wgpu/dx12".into(),
        browsers: vec![
            BrowserLine {
                name: "chrome".into(),
                version: Some("129.0".into()),
                extension_version: Some("1.4.2".into()),
                host_registered: true,
                last_ping: Some("2026-09-29T14:02Z".into()),
            },
            BrowserLine {
                name: "edge".into(),
                version: Some("129.0".into()),
                extension_version: None,
                host_registered: true,
                last_ping: None,
            },
            BrowserLine {
                name: "firefox".into(),
                version: Some("131.0".into()),
                extension_version: Some("1.4.2".into()),
                host_registered: true,
                last_ping: Some("2026-09-28T20:40Z".into()),
            },
        ],
        devices: vec![
            DeviceLine::Usb {
                vid_pid: "0529:0620".into(),
                hint_id: Some("safenet-etoken-5110".into()),
                certs: 0,
            },
            DeviceLine::Reader {
                name: "Identiv uTrust 2700 R".into(),
                hint_id: None,
                atr: Some("3B:D5:18:FF:81:91:FE:1F:C3:80:73:C8:21:10:0A".into()),
                certs: 1,
            },
        ],
        modules: vec![
            ModuleLine {
                path: "%ProgramFiles%\\OpenSC Project\\OpenSC\\pkcs11\\opensc-pkcs11.dll".into(),
                result: Ok((1, 1)),
                user_added: false,
            },
            ModuleLine {
                path: "%USERPROFILE%\\Downloads\\wdpkcs_icp.dll".into(),
                result: Err("file not found".into()),
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
            kinds: vec![("icp-brasil-a3".into(), 1), ("icp-brasil-a1".into(), 2)],
            keys: vec![("rsa-2048".into(), 3)],
            expiring_within_30_days: 2,
        },
        complement: "n/a".into(),
        recent_errors: vec![ErrorLine {
            at: "2026-09-29T14:05Z".into(),
            operation: "sign".into(),
            code: "PinIncorrect".into(),
            source: "pkcs11".into(),
            native: Some("CKR_PIN_INCORRECT".into()),
        }],
    }
}

const GOLDEN: &str = "\
SignLocal diagnostics v1
app: 1.4.0 (msix, x86_64) · protocol 1 · locale pt-BR · scale 125%
os: Windows 11 23H2 (10.0.22631)
render: wgpu/dx12
browsers:
  chrome 129.0 · extension 1.4.2 · host registered · last ping 2026-09-29T14:02Z
  edge 129.0 · extension not seen · host registered
  firefox 131.0 · extension 1.4.2 · host registered · last ping 2026-09-28T20:40Z
devices:
  usb 0529:0620 safenet-etoken-5110 · certs 0
  reader \"Identiv uTrust 2700 R\" · atr 3B:D5:18:FF:81:91:FE:1F:C3:..:..:..:..:..:.. · certs 1
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
fn matches_the_documented_example() {
    assert_eq!(render(&example()), GOLDEN);
}

#[test]
fn empty_sections_say_none_and_unknowns_have_placeholders() {
    let mut input = example();
    input.browsers = vec![BrowserLine {
        name: "safari".into(),
        ..Default::default()
    }];
    input.devices.clear();
    input.modules.clear();
    input.recent_errors.clear();
    input.certificates.hidden_other = 2;
    input.certificates.kinds.clear();
    let text = render(&input);
    assert!(text.contains("  safari ? · extension not seen · host not registered\n"));
    assert!(text.contains("devices:\n  none\npkcs11:\n  none\ncertificates:"));
    assert!(text.ends_with("recent errors (last 20):\n  none\n"));
    assert!(text.contains("hidden 4 (expired 1, login-only 1, other 2)"));
    assert!(text.contains("kinds: none · keys: rsa-2048 3"));
}

#[test]
fn only_the_newest_twenty_errors_and_a_card_is_shown_masked() {
    let mut input = example();
    input.recent_errors = (0..25)
        .map(|n| ErrorLine {
            at: format!("t{n}"),
            operation: "sign".into(),
            code: "Internal".into(),
            source: "app".into(),
            native: None,
        })
        .collect();
    input.devices = vec![DeviceLine::Reader {
        name: "R".into(),
        hint_id: None,
        atr: Some("3b00".into()),
        certs: 0,
    }];
    let text = render(&input);
    assert!(!text.contains("  t4 sign"));
    assert!(text.contains("  t5 sign Internal app\n"));
    assert!(text.ends_with("  t24 sign Internal app\n"));
    assert!(text.contains("atr 3B:00 · certs 0"));
}
