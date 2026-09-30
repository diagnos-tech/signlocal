//! Certificates for the Certificates tab: every key source's listing,
//! de-duplicated, as the rows of the confirmation window's list
//! (`docs/ux.md` §8.5), with "Can't sign" rows kept.

use std::collections::HashMap;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use websign_core::{Fingerprint, SourceKind};
use websign_devices::hints::{DeviceDatabase, DeviceKind};
use websign_keystores::inventory::Inventory;
use websign_keystores::{DeviceLink, FoundKey};
use websign_ui_model::certs::{
    CertCandidate, DeviceLabel, KeySource, ListContext, PinMode, build_cert_list,
};

use super::CertificatesFact;
use super::hidden_rows::hidden_rows;

/// The list built from `inventory` at `now`, in `zone`.
pub fn collect(
    inventory: &Inventory,
    hints: Option<&DeviceDatabase>,
    now: Timestamp,
    zone: &TimeZone,
) -> CertificatesFact {
    let mut candidates = Vec::new();
    let mut der = HashMap::new();
    let mut deduplicated = 0;
    for group in inventory.groups() {
        let Some(entry) = inventory.entries.get(group.primary) else {
            continue;
        };
        deduplicated += u32::try_from(group.alternates.len()).unwrap_or(u32::MAX);
        let fingerprint = Fingerprint::of(&entry.key.cert_der);
        let alternates = group
            .alternates
            .iter()
            .filter_map(|&index| inventory.entries.get(index))
            .map(|alternate| websign_ui_model::certs::KeyPath {
                source: key_source(&alternate.key),
                pin: PinMode::System,
            })
            .collect();
        candidates.push(CertCandidate {
            fingerprint,
            info: entry.info.clone(),
            source: key_source(&entry.key),
            alternates,
            device: device_label(entry.key.device.as_ref(), hints),
            // Diagnostics never signs: who asks for the PIN does not matter.
            pin: PinMode::System,
            algorithms: Vec::new(),
            hardware: entry.key.hardware,
            has_private_key: true,
            removed: false,
        });
        der.insert(fingerprint, entry.key.cert_der.clone());
    }
    from_candidates(&candidates, der, deduplicated, now, zone)
}

/// The tab's list of `candidates` at `now`: the confirmation window's rules,
/// plus the hidden certificates worth a row.
pub fn from_candidates(
    candidates: &[CertCandidate],
    der: HashMap<Fingerprint, Vec<u8>>,
    deduplicated: u32,
    now: Timestamp,
    zone: &TimeZone,
) -> CertificatesFact {
    let context = list_context(now, zone);
    let list = build_cert_list(candidates, &context);
    CertificatesFact {
        hidden_rows: hidden_rows(candidates, &list, &context),
        list,
        der,
        deduplicated,
    }
}

/// The context of a list that accepts every algorithm and prefers nothing:
/// the tab shows what exists, not what one site asked for.
fn list_context(now: Timestamp, zone: &TimeZone) -> ListContext {
    ListContext {
        today: now.to_zoned(zone.clone()).date(),
        time_zone: zone.clone(),
        now: now.as_second(),
        accepted: Vec::new(),
        last_used_here: None,
        recent_anywhere: Vec::new(),
        requested: None,
    }
}

/// The path a key is reached through.
fn key_source(key: &FoundKey) -> KeySource {
    match (key.kind, key.keystore.as_str()) {
        (SourceKind::Pkcs11, name) => KeySource::Driver {
            path: name.strip_prefix("pkcs11:").unwrap_or(name).to_owned(),
        },
        (SourceKind::System, "macos:keychain") => KeySource::MacosKeychain,
        (SourceKind::System, "macos:ctk") => KeySource::MacosToken,
        (SourceKind::System, _) => KeySource::Windows,
    }
}

/// The device a key lives on, named from `devices.json` when the token's
/// model matches an entry; never a serial number or a token label.
fn device_label(link: Option<&DeviceLink>, hints: Option<&DeviceDatabase>) -> Option<DeviceLabel> {
    match link? {
        DeviceLink::Reader { name } => Some(DeviceLabel::CardInReader {
            reader: name.clone(),
        }),
        DeviceLink::Pkcs11Token { model, .. } => {
            let model = model.trim().to_lowercase();
            let known = hints.and_then(|db| {
                db.devices.iter().find(|hint| {
                    hint.kind == DeviceKind::Token
                        && !model.is_empty()
                        && hint.name.to_lowercase().contains(&model)
                })
            });
            Some(match known {
                Some(hint) => DeviceLabel::Token {
                    name: hint.name.clone(),
                },
                None => DeviceLabel::Unknown,
            })
        }
        DeviceLink::CryptoTokenKit { .. } => Some(DeviceLabel::Unknown),
    }
}
