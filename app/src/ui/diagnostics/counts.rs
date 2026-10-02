//! Certificate counts for the report (`docs/ux.md` §8.7): numbers and
//! categories only, taken from the same list the Certificates tab shows.

use std::cmp::Reverse;
use std::collections::BTreeMap;

use jiff::Timestamp;
use websign_core::PublicKeyKind;
use websign_ui_model::certs::{Badge, CertRow, DisabledReason, HiddenReason, KeySource, RowStatus};
use websign_ui_model::diagnostics::report::CertificateCounts;

use super::facts::CertificatesFact;

const THIRTY_DAYS: i64 = 30 * 24 * 60 * 60;

/// The counts of `fact` at `now`.
pub fn certificate_counts(fact: &CertificatesFact, now: Timestamp) -> CertificateCounts {
    let list = &fact.list;
    let mut counts = CertificateCounts {
        deduplicated: fact.deduplicated,
        ..CertificateCounts::default()
    };
    for row in &list.usable {
        match row.candidate.source {
            KeySource::Driver { .. } => counts.usable_pkcs11 += 1,
            _ => counts.usable_os += 1,
        }
        let expiring = row
            .candidate
            .info
            .as_ref()
            .is_ok_and(|info| info.not_after - now.as_second() <= THIRTY_DAYS);
        counts.expiring_within_30_days += u32::from(expiring);
    }
    for row in &list.disabled {
        match row.status {
            RowStatus::Disabled(DisabledReason::Expired) => counts.hidden_expired += 1,
            _ => counts.hidden_other += 1,
        }
    }
    for (_, reason) in &list.hidden {
        match reason {
            HiddenReason::KeyUsage
            | HiddenReason::ExtendedKeyUsage
            | HiddenReason::LoginSibling => {
                counts.hidden_login_only += 1;
            }
            _ => counts.hidden_other += 1,
        }
    }
    counts.kinds = tally(list.usable.iter().map(|row| kind(&row.badge)), kind_rank);
    counts.keys = tally(list.usable.iter().filter_map(key), |_| ());
    counts
}

/// `(name, count)` pairs ordered by `rank` (then by count, larger first).
fn tally<K: Ord>(
    names: impl Iterator<Item = String>,
    rank: impl Fn(&str) -> K,
) -> Vec<(String, u32)> {
    let mut counted: BTreeMap<String, u32> = BTreeMap::new();
    for name in names {
        *counted.entry(name).or_default() += 1;
    }
    let mut pairs: Vec<(String, u32)> = counted.into_iter().collect();
    pairs.sort_by_key(|(name, count)| (rank(name), Reverse(*count)));
    pairs
}

/// `icp-brasil-a3`, `eidas-qualified`, `pt-cc`, `other`.
fn kind(badge: &Badge) -> String {
    match badge {
        Badge::IcpBrasil { class } => format!("icp-brasil-{}", class.to_lowercase()),
        Badge::IcpBrasilPlain => "icp-brasil".to_owned(),
        Badge::EidasQualified => "eidas-qualified".to_owned(),
        Badge::Eidas => "eidas".to_owned(),
        Badge::PtCitizenCard => "pt-cc".to_owned(),
        Badge::EsDnie => "es-dnie".to_owned(),
        Badge::Generic => "other".to_owned(),
    }
}

/// Stronger kinds first: ICP-Brasil by class (A3 before A1), then the
/// eIDAS and national cards, `other` last.
fn kind_rank(name: &str) -> (u8, Reverse<String>) {
    let family = match name {
        _ if name.starts_with("icp-brasil") => 0,
        "eidas-qualified" => 1,
        "eidas" => 2,
        "pt-cc" | "es-dnie" => 3,
        _ => 4,
    };
    (family, Reverse(name.to_owned()))
}

/// `rsa-2048`, `ec-p-256`.
fn key(row: &CertRow) -> Option<String> {
    match &row.candidate.info.as_ref().ok()?.key {
        PublicKeyKind::Rsa { bits } => Some(format!("rsa-{bits}")),
        PublicKeyKind::Ec { curve } => Some(format!("ec-{}", curve.name().to_ascii_lowercase())),
        PublicKeyKind::Unsupported { .. } => Some("unsupported".to_owned()),
    }
}
