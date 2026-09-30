//! Report inputs: the example of ux §8.7, field by field.

use websign_ui_model::diagnostics::report::{
    BrowserLine, CertificateCounts, DeviceLine, ErrorLine, ModuleLine, ReportInput,
};

pub const ATR: &str = "3B:D5:18:FF:81:91:FE:1F:C3:80:73:C8:21:10:0A";
pub const ATR_MASKED: &str = "3B:D5:18:FF:81:91:FE:1F:C3:..:..:..:..:..:..";

pub fn s(text: &str) -> String {
    text.to_owned()
}

/// The example of ux §8.7, field by field.
pub fn example() -> ReportInput {
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
                hint_id: None,
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

/// The example with every list section empty.
pub fn empty_sections() -> ReportInput {
    ReportInput {
        browsers: vec![],
        devices: vec![],
        modules: vec![],
        recent_errors: vec![],
        ..example()
    }
}
