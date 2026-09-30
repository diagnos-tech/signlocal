//! Certificates, candidates and list contexts.

use jiff::civil::{Date, date};
use jiff::tz::TimeZone;
use websign_core::{
    CertInfo, DistinguishedName, Fingerprint, KeyUsage, PublicKeyKind, SignatureAlgorithm,
};
use websign_ui_model::certs::{
    CertCandidate, CertList, CertRow, DeviceLabel, HiddenReason, KeySource, ListContext, PinMode,
    RowStatus,
};

pub const SERVER_AUTH: &str = "1.3.6.1.5.5.7.3.1";
pub const CLIENT_AUTH: &str = "1.3.6.1.5.5.7.3.2";
pub const CODE_SIGNING: &str = "1.3.6.1.5.5.7.3.3";
pub const EMAIL_PROTECTION: &str = "1.3.6.1.5.5.7.3.4";
pub const TIME_STAMPING: &str = "1.3.6.1.5.5.7.3.8";
pub const OCSP_SIGNING: &str = "1.3.6.1.5.5.7.3.9";
pub const DOCUMENT_SIGNING: &str = "1.3.6.1.5.5.7.3.36";
pub const MS_DOCUMENT_SIGNING: &str = "1.3.6.1.4.1.311.10.3.12";
pub const ANY_EKU: &str = "2.5.29.37.0";

/// A fingerprint made of one repeated byte.
pub fn fp(n: u8) -> Fingerprint {
    Fingerprint::from_bytes([n; 32])
}

/// Local "today" of every list test.
pub fn today() -> Date {
    date(2026, 9, 29)
}

/// Unix seconds of noon UTC on the given date (the test context uses UTC).
pub fn noon(year: i16, month: i8, day: i8) -> i64 {
    date(year, month, day)
        .at(12, 0, 0, 0)
        .to_zoned(TimeZone::UTC)
        .unwrap()
        .timestamp()
        .as_second()
}

/// "Now" of every list test: today at noon.
pub fn now() -> i64 {
    noon(2026, 9, 29)
}

/// A signing certificate (digitalSignature + nonRepudiation, RSA 2048)
/// valid 2025-01-01 to 2027-01-01, held by `name`, issued by "AC Test".
pub fn info(n: u8, name: &str) -> CertInfo {
    CertInfo {
        fingerprint: fp(n),
        subject: DistinguishedName {
            common_name: Some(name.to_owned()),
            ..DistinguishedName::default()
        },
        issuer: DistinguishedName {
            common_name: Some("AC Test".to_owned()),
            ..DistinguishedName::default()
        },
        serial_hex: format!("{n:02x}"),
        not_before: noon(2025, 1, 1),
        not_after: noon(2027, 1, 1),
        key: PublicKeyKind::Rsa { bits: 2048 },
        key_usage: Some(KeyUsage {
            digital_signature: true,
            non_repudiation: true,
            ..KeyUsage::default()
        }),
        extended_key_usage: Vec::new(),
        policies: Vec::new(),
        is_ca: false,
        icp_brasil: None,
        qualified: None,
    }
}

/// A usable candidate: software key in the Windows store, RSA algorithms, a
/// private key, System PIN.
pub fn candidate(n: u8, name: &str) -> CertCandidate {
    from_info(info(n, name))
}

pub fn from_info(info: CertInfo) -> CertCandidate {
    CertCandidate {
        fingerprint: info.fingerprint,
        info: Ok(info),
        source: KeySource::Windows,
        alternates: Vec::new(),
        device: None,
        pin: PinMode::System,
        algorithms: vec![SignatureAlgorithm::RsaPkcs1v15, SignatureAlgorithm::RsaPss],
        hardware: Some(false),
        has_private_key: true,
        removed: false,
    }
}

/// Edits the certificate summary of `candidate`.
pub fn with_info(mut candidate: CertCandidate, edit: impl FnOnce(&mut CertInfo)) -> CertCandidate {
    let mut info = candidate.info.clone().unwrap();
    edit(&mut info);
    candidate.info = Ok(info);
    candidate
}

/// Two software certificates; Ana (1) sorts first, Bia (2) second.
pub fn pair() -> Vec<CertCandidate> {
    vec![candidate(1, "Ana"), candidate(2, "Bia")]
}

/// A token key in a PKCS#11 driver that asks for our PIN (4 to 16 chars).
pub fn pin_token(n: u8, name: &str, pin: PinMode) -> CertCandidate {
    let mut c = candidate(n, name);
    c.source = driver("eTPKCS11.dll");
    c.device = token("SafeNet eToken 5110");
    c.hardware = Some(true);
    c.pin = pin;
    c
}

pub fn token(name: &str) -> Option<DeviceLabel> {
    Some(DeviceLabel::Token {
        name: name.to_owned(),
    })
}

pub fn driver(path: &str) -> KeySource {
    KeySource::Driver {
        path: path.to_owned(),
    }
}

pub fn app_pin(length: Option<(u32, u32)>) -> PinMode {
    PinMode::App {
        length,
        count_low: false,
        final_try: false,
        locked: false,
    }
}

/// Context with no history, any algorithm, at [`today`].
pub fn context() -> ListContext {
    ListContext {
        today: today(),
        time_zone: TimeZone::UTC,
        now: now(),
        accepted: Vec::new(),
        last_used_here: None,
        recent_anywhere: Vec::new(),
        requested: None,
    }
}

pub fn fps(rows: &[CertRow]) -> Vec<Fingerprint> {
    rows.iter().map(|row| row.candidate.fingerprint).collect()
}

pub fn usable(list: &CertList) -> Vec<Fingerprint> {
    fps(&list.usable)
}

pub fn disabled(list: &CertList) -> Vec<Fingerprint> {
    fps(&list.disabled)
}

/// The row of `fingerprint`, wherever it sits.
pub fn row(list: &CertList, fingerprint: Fingerprint) -> &CertRow {
    list.usable
        .iter()
        .chain(&list.disabled)
        .find(|row| row.candidate.fingerprint == fingerprint)
        .expect("row is listed")
}

pub fn status(list: &CertList, fingerprint: Fingerprint) -> RowStatus {
    row(list, fingerprint).status
}

pub fn hidden_reason(list: &CertList, fingerprint: Fingerprint) -> Option<HiddenReason> {
    list.hidden
        .iter()
        .find(|(f, _)| *f == fingerprint)
        .map(|(_, reason)| *reason)
}
