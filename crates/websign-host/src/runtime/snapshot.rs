//! From what the key sources found to what the engine and the window list.

use websign_core::present::wire::signature_algorithm;
use websign_core::{Fingerprint, PublicKeyKind, SignatureAlgorithm, SourceKind};
use websign_devices::hints::DeviceDatabase;
use websign_keystores::{FoundKey, KeyRef, KeystoreHub, PinPrompt};
use websign_protocol::types::SignatureAlgorithmName;
use websign_ui_model::certs::{CertCandidate, KeyPath, KeySource, PinMode};

use super::device_label::device_label;
use super::possible::{linked_devices, possible_cards};
use crate::ports::KeySnapshot;

/// One certificate as the sources reported it, before the hub is asked
/// about PIN states (which needs the hub again, so the listing's borrow must
/// end first).
struct Found {
    fingerprint: Fingerprint,
    der: Vec<u8>,
    info: Result<websign_core::CertInfo, websign_core::CertError>,
    primary: FoundKey,
    alternates: Vec<FoundKey>,
}

/// The listing of `hub` as candidates: one per certificate, the OS path
/// first, with device labels from `devices.json` and PIN modes from the
/// tokens' own flags (read without logging in, for every path: a driver
/// that sees an OS store's key needs our PIN field when signing through
/// it); plus the devices of `scan` that brought no certificate.
pub(super) fn snapshot(
    hub: &mut KeystoreHub,
    hints: Option<&DeviceDatabase>,
    scan: &websign_devices::Snapshot,
) -> KeySnapshot {
    let (found, failures) = collect(hub);
    let linked = linked_devices(found.iter().map(|group| &group.primary));
    let mut snapshot = KeySnapshot {
        failures,
        possible: hints
            .map(|database| possible_cards(scan, database, &linked))
            .unwrap_or_default(),
        ..KeySnapshot::default()
    };
    for group in found {
        let pin = pin_mode(hub, group.fingerprint, 0, &group.primary);
        let alternates = group
            .alternates
            .iter()
            .enumerate()
            .map(|(index, key)| KeyPath {
                source: key_source(key),
                pin: pin_mode(hub, group.fingerprint, index + 1, key),
            })
            .collect();
        let algorithms = group
            .info
            .as_ref()
            .map(|info| algorithms_of(&info.key))
            .unwrap_or_default();
        snapshot.candidates.push(CertCandidate {
            fingerprint: group.fingerprint,
            info: group.info,
            source: key_source(&group.primary),
            alternates,
            device: device_label(group.primary.device.as_ref(), hints),
            pin,
            algorithms,
            hardware: group.primary.hardware,
            has_private_key: true,
            removed: false,
        });
        snapshot.certificates.insert(group.fingerprint, group.der);
    }
    snapshot
}

fn collect(hub: &mut KeystoreHub) -> (Vec<Found>, Vec<String>) {
    let inventory = hub.inventory();
    let mut found = Vec::new();
    for group in inventory.groups() {
        let Some(entry) = inventory.entries.get(group.primary) else {
            continue;
        };
        let alternates = group
            .alternates
            .iter()
            .filter_map(|&index| inventory.entries.get(index))
            .map(|entry| entry.key.clone())
            .collect();
        found.push(Found {
            fingerprint: Fingerprint::of(&entry.key.cert_der),
            der: entry.key.cert_der.clone(),
            info: entry.info.clone(),
            primary: entry.key.clone(),
            alternates,
        });
    }
    (found, inventory.warnings())
}

/// What the key store can produce with a key of this type; a store that
/// cannot (RSASSA-PSS through legacy CAPI) reports `Unsupported` when
/// signing and the row is then marked incompatible.
fn algorithms_of(key: &PublicKeyKind) -> Vec<SignatureAlgorithm> {
    let names: &[SignatureAlgorithmName] = match key {
        PublicKeyKind::Rsa { .. } => &[
            SignatureAlgorithmName::RsaPkcs1v15,
            SignatureAlgorithmName::RsaPss,
        ],
        PublicKeyKind::Ec { .. } => &[SignatureAlgorithmName::Ecdsa],
        PublicKeyKind::Unsupported { .. } => &[],
    };
    names.iter().copied().map(signature_algorithm).collect()
}

/// The path the signature takes, from the source's stable name.
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

/// Who asks for the PIN on path `path` of the key (`KeyRef::path`).
fn pin_mode(
    hub: &mut KeystoreHub,
    fingerprint: Fingerprint,
    path: usize,
    key: &FoundKey,
) -> PinMode {
    match key.pin {
        PinPrompt::System => PinMode::System,
        PinPrompt::App {
            protected_path: true,
        } => PinMode::PinPad,
        PinPrompt::App { .. } => {
            let state = hub.pin_state(KeyRef { fingerprint, path });
            match state {
                Some(state) if state.unlocked && !state.always_authenticate => PinMode::Unlocked,
                Some(state) => PinMode::App {
                    length: state.length,
                    count_low: state.count_low,
                    final_try: state.final_try,
                    locked: state.locked,
                },
                None => PinMode::App {
                    length: None,
                    count_low: false,
                    final_try: false,
                    locked: false,
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algorithms_follow_the_key_type() {
        assert_eq!(
            algorithms_of(&PublicKeyKind::Rsa { bits: 2048 }),
            [SignatureAlgorithm::RsaPkcs1v15, SignatureAlgorithm::RsaPss]
        );
        assert!(algorithms_of(&PublicKeyKind::Unsupported { oid: "1.2".into() }).is_empty());
    }
}
