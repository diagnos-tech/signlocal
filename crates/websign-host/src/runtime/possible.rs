//! "Possible certificates" for the window (`docs/ux.md` §6): tokens and
//! cards that are plugged in but brought no certificate, usually because
//! their driver is missing.

use websign_devices::Snapshot;
use websign_devices::hints::{DeviceDatabase, DeviceKind, PerOs};
use websign_devices::possible::{LinkedDevices, PossibleDevice, PossibleSource, possible_devices};
use websign_keystores::{DeviceLink, FoundKey};
use websign_ui_model::possible::PossibleCard;

/// What the listed keys say about their devices, so a device that already
/// brought a certificate is not suggested again.
pub(super) fn linked_devices<'a>(keys: impl IntoIterator<Item = &'a FoundKey>) -> LinkedDevices {
    let mut linked = LinkedDevices::default();
    for key in keys {
        match &key.device {
            Some(DeviceLink::Reader { name }) => linked.readers.push(name.clone()),
            Some(DeviceLink::Pkcs11Token { model, .. }) => linked.token_models.push(model.clone()),
            Some(DeviceLink::CryptoTokenKit { .. }) => linked.unknown_links = true,
            None if key.hardware == Some(true) => linked.unknown_links = true,
            None => {}
        }
    }
    linked
}

/// The devices of `scan` the confirmation window may suggest: only the
/// confident ones (a known model, and every listed hardware key accounted
/// for). The rest appear in diagnostics only.
pub(super) fn possible_cards(
    scan: &Snapshot,
    database: &DeviceDatabase,
    linked: &LinkedDevices,
) -> Vec<PossibleCard> {
    possible_devices(scan, database, linked)
        .into_iter()
        .filter(|device| device.confident)
        .filter_map(card)
        .collect()
}

fn card(device: PossibleDevice) -> Option<PossibleCard> {
    let hint = device.hint?;
    let is_card =
        hint.kind == DeviceKind::Card || matches!(device.source, PossibleSource::Card { .. });
    let driver = hint.driver.as_ref();
    Some(PossibleCard {
        name: Some(hint.name.clone()),
        // Only an unknown card is described by its reader, and unknown cards
        // are never confident.
        reader: None,
        card: is_card,
        driver: driver.map(|driver| driver.name.clone()),
        download: driver.and_then(|driver| this_os(&driver.download).cloned()),
        // Direct builds are not sandboxed, so drivers load without the
        // store build's Add-on (`docs/plan.md` D10).
        needs_complement: false,
    })
}

fn this_os<T>(values: &PerOs<T>) -> Option<&T> {
    if cfg!(windows) {
        values.windows.as_ref()
    } else if cfg!(target_os = "macos") {
        values.macos.as_ref()
    } else {
        values.linux.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use websign_core::SourceKind;
    use websign_devices::pcsc::ReaderScan;
    use websign_devices::usb::{UsbDevice, UsbScan};
    use websign_keystores::PinPrompt;

    use super::*;

    fn key(device: Option<DeviceLink>, hardware: Option<bool>) -> FoundKey {
        FoundKey {
            cert_der: Vec::new(),
            keystore: "pkcs11:libx.so".to_owned(),
            kind: SourceKind::Pkcs11,
            provider: String::new(),
            hardware,
            locator: String::new(),
            pin: PinPrompt::System,
            device,
        }
    }

    fn scan(product: &str) -> Snapshot {
        Snapshot {
            usb: UsbScan {
                devices: vec![UsbDevice {
                    vendor_id: 0x0529,
                    product_id: 0x0620,
                    manufacturer: None,
                    product: Some(product.to_owned()),
                    classes: vec![0x0B],
                    smart_card: true,
                }],
                hidden: 0,
                problem: None,
            },
            readers: ReaderScan {
                readers: Vec::new(),
                problem: None,
            },
        }
    }

    fn database() -> DeviceDatabase {
        DeviceDatabase::embedded().unwrap()
    }

    #[test]
    fn a_known_token_without_certificates_is_suggested_with_its_driver() {
        let cards = possible_cards(&scan("eToken"), &database(), &LinkedDevices::default());
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].name.as_deref(), Some("SafeNet eToken 5110"));
        assert!(!cards[0].card && cards[0].driver.is_some());
    }

    #[test]
    fn a_token_that_brought_a_certificate_is_not_suggested() {
        let reader = DeviceLink::Reader {
            name: "SafeNet eToken 5110 [eToken] 00 00".to_owned(),
        };
        let linked = linked_devices(&[key(Some(reader), Some(true))]);
        assert!(possible_cards(&scan("eToken"), &database(), &linked).is_empty());
    }

    #[test]
    fn a_hardware_key_on_an_unknown_device_silences_every_suggestion() {
        let linked = linked_devices(&[key(None, Some(true)), key(None, Some(false))]);
        assert!(linked.unknown_links);
        assert!(possible_cards(&scan("eToken"), &database(), &linked).is_empty());
        let software_only = linked_devices(&[key(None, Some(false))]);
        assert!(!software_only.unknown_links);
    }
}
