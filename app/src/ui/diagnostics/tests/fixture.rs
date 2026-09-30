//! The scenario of the mockups and of `docs/ux.md` §8.7: Windows, a new
//! token without its driver, a card in a reader, one driver that failed,
//! two certificates about to expire. Built from the same types a scan
//! fills, so the report and the tabs go through the real code.

use std::collections::HashMap;
use std::path::PathBuf;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use websign_core::{
    CertInfo, DistinguishedName, Fingerprint, IcpBrasil, IcpLevel, KeyUsage, PublicKeyKind,
};
use websign_host::store::{ConnectionRecord, ErrorRecord};
use websign_protocol::types::BrowserName;
use websign_registration::Browser;
use websign_registration::detect::BrowserPackaging;
use websign_registration::status::RegistrationState;
use websign_ui_model::certs::{CertCandidate, DeviceLabel, KeySource, PinMode};

use crate::ui::diagnostics::facts::{
    BrowserFact, CardFact, CertPresence, DevicesFact, DriverFact, Facts, HintFact, ReaderFact,
    TokenFact, from_candidates,
};
use crate::ui::diagnostics::report::Environment;

/// 2026-09-29 14:05 UTC: three minutes after Chrome's last ping.
pub fn now() -> Timestamp {
    Timestamp::from_second(1_790_690_700).unwrap_or(Timestamp::UNIX_EPOCH)
}

/// Unix seconds of a UTC date and time.
pub fn at(date: &str) -> i64 {
    date.parse::<Timestamp>().map_or(0, |t| t.as_second())
}

pub fn environment() -> Environment {
    Environment {
        app_version: "1.4.0".to_owned(),
        packaging: "msix".to_owned(),
        arch: "x86_64".to_owned(),
        locale: "pt-BR".to_owned(),
        scale_percent: 125,
        render: "wgpu/dx12".to_owned(),
        complement: "n/a".to_owned(),
    }
}

pub fn facts() -> Facts {
    Facts {
        browsers: vec![
            browser(
                Browser::Chrome,
                "129.0.6668.59",
                Some(at("2026-09-29T14:02:00Z")),
            ),
            browser(Browser::Edge, "129.0.2792.52", None),
            browser(Browser::Firefox, "131.0", Some(at("2026-09-28T20:40:00Z"))),
        ],
        devices: devices(),
        drivers: drivers(),
        certificates: from_candidates(&certificates(), HashMap::new(), 1, now(), &TimeZone::UTC),
        recent_errors: vec![ErrorRecord {
            at: at("2026-09-29T14:05:00Z"),
            operation: "sign".to_owned(),
            code: "PinIncorrect".to_owned(),
            source: "pkcs11".to_owned(),
            native: Some("CKR_PIN_INCORRECT".to_owned()),
        }],
        os: "Windows 11 23H2 (10.0.22631)".to_owned(),
    }
}

fn browser(browser: Browser, version: &str, last_seen: Option<i64>) -> BrowserFact {
    let name = match browser {
        Browser::Edge => BrowserName::Edge,
        Browser::Firefox => BrowserName::Firefox,
        _ => BrowserName::Chrome,
    };
    BrowserFact {
        browser,
        version: Some(version.to_owned()),
        packaging: BrowserPackaging::Native,
        registration: RegistrationState::Registered,
        connection: last_seen.map(|last_seen| ConnectionRecord {
            browser: name,
            browser_version: version.to_owned(),
            extension_version: "1.4.2".to_owned(),
            last_seen,
        }),
    }
}

fn devices() -> DevicesFact {
    DevicesFact {
        pcscd_running: None,
        tokens: vec![TokenFact {
            vid_pid: "0529:0620".to_owned(),
            hint: Some(HintFact {
                id: "safenet-etoken-5110".to_owned(),
                name: "SafeNet eToken 5110".to_owned(),
                driver: Some("SafeNet Authentication Client".to_owned()),
                download: Some("https://example.com/sac".to_owned()),
            }),
            certificates: CertPresence::Missing,
        }],
        readers: vec![ReaderFact {
            name: "Identiv uTrust 2700 R".to_owned(),
            card: Some(CardFact {
                atr: Some("3BD518FF8191FE1FC38073C821100A".to_owned()),
                hint: None,
                certificates: CertPresence::Found(1),
            }),
        }],
    }
}

fn drivers() -> Vec<DriverFact> {
    let added = PathBuf::from(r"C:\Users\ana\Downloads\wdpkcs_icp.dll");
    vec![
        DriverFact {
            name: "opensc-pkcs11.dll".to_owned(),
            path: r"%ProgramFiles%\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll".to_owned(),
            result: Ok(1),
            user_added: false,
            setting: None,
        },
        DriverFact {
            name: "wdpkcs_icp.dll".to_owned(),
            path: r"%USERPROFILE%\Downloads\wdpkcs_icp.dll".to_owned(),
            result: Err("file not found".to_owned()),
            user_added: true,
            setting: Some(added),
        },
    ]
}

fn certificates() -> Vec<CertCandidate> {
    let reader = DeviceLabel::CardInReader {
        reader: "Identiv uTrust 2700 R".to_owned(),
    };
    let mut a3 = candidate(
        1,
        "ANA BEATRIZ SOUZA:12345678901",
        IcpLevel::A3,
        "2026-10-22T12:00:00Z",
    );
    a3.device = Some(reader.clone());
    a3.hardware = Some(true);
    a3.alternates = vec![KeySource::Driver {
        path: "opensc-pkcs11.dll".to_owned(),
    }];
    set_issuer(&mut a3, "AC SOLUTI Multipla v5");
    let a1 = candidate(
        2,
        "ANA BEATRIZ SOUZA:12345678901",
        IcpLevel::A1,
        "2027-03-14T12:00:00Z",
    );
    let mut company = candidate(
        3,
        "CLINICA SOUZA IMAGEM LTDA:12345678000190",
        IcpLevel::A1,
        "2026-10-04T12:00:00Z",
    );
    if let Ok(info) = &mut company.info
        && let Some(icp) = &mut info.icp_brasil
    {
        icp.cpf = None;
        icp.cnpj = Some("12345678000190".to_owned());
        icp.holder_name = Some("CLINICA SOUZA IMAGEM LTDA".to_owned());
    }
    let mut expired = candidate(
        4,
        "ANA BEATRIZ SOUZA:12345678901",
        IcpLevel::A3,
        "2026-05-10T12:00:00Z",
    );
    expired.device = Some(reader);
    expired.hardware = Some(true);
    set_issuer(&mut expired, "AC SOLUTI Multipla v5");
    let mut login = candidate(
        5,
        "Ana B. Souza - VPN Clinica",
        IcpLevel::A1,
        "2027-01-20T12:00:00Z",
    );
    if let Ok(info) = &mut login.info {
        info.icp_brasil = None;
        info.key_usage = Some(KeyUsage {
            key_encipherment: true,
            ..KeyUsage::default()
        });
        info.issuer.common_name = Some("AC Interna Clinica Souza".to_owned());
    }
    vec![a3, a1, company, expired, login]
}

fn set_issuer(candidate: &mut CertCandidate, issuer: &str) {
    if let Ok(info) = &mut candidate.info {
        info.issuer.common_name = Some(issuer.to_owned());
    }
}

/// A software ICP-Brasil certificate in the Windows store.
fn candidate(seed: u8, cn: &str, level: IcpLevel, not_after: &str) -> CertCandidate {
    let holder = cn.split(':').next().unwrap_or(cn).to_owned();
    let info = CertInfo {
        fingerprint: Fingerprint::from_bytes([seed; 32]),
        subject: DistinguishedName {
            common_name: Some(cn.to_owned()),
            ..DistinguishedName::default()
        },
        issuer: DistinguishedName {
            common_name: Some("AC Certisign RFB G5".to_owned()),
            ..DistinguishedName::default()
        },
        serial_hex: format!("{seed:02x}"),
        not_before: at("2024-01-01T00:00:00Z"),
        not_after: at(not_after),
        key: PublicKeyKind::Rsa { bits: 2048 },
        key_usage: Some(KeyUsage {
            digital_signature: true,
            non_repudiation: true,
            ..KeyUsage::default()
        }),
        extended_key_usage: Vec::new(),
        policies: Vec::new(),
        is_ca: false,
        icp_brasil: Some(IcpBrasil {
            level: Some(level),
            holder_name: Some(holder),
            cpf: Some("12345678901".to_owned()),
            cnpj: None,
        }),
        qualified: None,
    };
    CertCandidate {
        fingerprint: info.fingerprint,
        info: Ok(info),
        source: KeySource::Windows,
        alternates: Vec::new(),
        device: None,
        pin: PinMode::System,
        algorithms: Vec::new(),
        hardware: Some(false),
        has_private_key: true,
        removed: false,
    }
}
