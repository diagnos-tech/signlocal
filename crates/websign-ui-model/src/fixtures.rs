//! Test candidates and contexts shared by the unit tests.

use jiff::civil::date;
use websign_core::{
    CertInfo, DistinguishedName, Fingerprint, IcpBrasil, IcpLevel, KeyUsage, PublicKeyKind,
    SignatureAlgorithm,
};

use crate::certs::{CertCandidate, KeySource, ListContext, PinMode};

/// 2026-09-29 12:00 UTC.
pub const NOW: i64 = 1_790_683_200;

pub fn fingerprint(seed: u8) -> Fingerprint {
    Fingerprint::from_bytes([seed; 32])
}

pub fn info(seed: u8, cn: &str) -> CertInfo {
    CertInfo {
        fingerprint: fingerprint(seed),
        subject: DistinguishedName {
            common_name: Some(cn.to_owned()),
            ..Default::default()
        },
        issuer: DistinguishedName {
            common_name: Some("AC Test".to_owned()),
            ..Default::default()
        },
        serial_hex: format!("{seed:02x}"),
        not_before: 1_577_836_800,
        not_after: 1_893_456_000,
        key: PublicKeyKind::Rsa { bits: 2048 },
        key_usage: Some(KeyUsage {
            digital_signature: true,
            non_repudiation: true,
            ..Default::default()
        }),
        extended_key_usage: Vec::new(),
        policies: Vec::new(),
        is_ca: false,
        icp_brasil: Some(IcpBrasil {
            level: Some(IcpLevel::A3),
            ..Default::default()
        }),
        qualified: None,
    }
}

/// A usable hardware certificate reached through the Windows store.
pub fn candidate(seed: u8, cn: &str) -> CertCandidate {
    CertCandidate {
        fingerprint: fingerprint(seed),
        info: Ok(info(seed, cn)),
        source: KeySource::Windows,
        alternates: Vec::new(),
        device: None,
        pin: PinMode::System,
        algorithms: vec![SignatureAlgorithm::RsaPkcs1v15],
        hardware: Some(true),
        has_private_key: true,
        removed: false,
    }
}

pub fn context() -> ListContext {
    ListContext {
        today: date(2026, 9, 29),
        now: NOW,
        accepted: Vec::new(),
        last_used_here: None,
        recent_anywhere: Vec::new(),
        requested: None,
    }
}
