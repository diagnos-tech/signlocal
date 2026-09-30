//! Tokens, cards and readers (`docs/ux.md` §8.4), and whether each brought
//! certificates: a device linked to a listed key has them; one of the
//! "possible certificates" (§6.1) has none.

use websign_devices::Snapshot;
use websign_devices::anonymous_reader_name;
use websign_devices::hints::{DeviceDatabase, DeviceHint, DeviceKind, PerOs};
use websign_devices::pcsc::{CardState, Reader};
use websign_devices::possible::{LinkedDevices, PossibleSource, possible_devices};
use websign_devices::usb::UsbDevice;
use websign_keystores::{DeviceLink, FoundKey};

use super::{CardFact, CertPresence, DevicesFact, HintFact, ReaderFact, TokenFact};

/// The devices of `scan`, with certificate counts from `keys`.
pub fn collect(scan: &Snapshot, hints: Option<&DeviceDatabase>, keys: &[&FoundKey]) -> DevicesFact {
    let linked = linked_devices(keys);
    let possible = hints
        .map(|db| possible_devices(scan, db, &linked))
        .unwrap_or_default();
    let presence = |source: &PossibleSource, linked_keys: u32| {
        if linked_keys > 0 {
            return CertPresence::Found(linked_keys);
        }
        match possible.iter().find(|device| device.source == *source) {
            Some(device) if device.confident => CertPresence::Missing,
            _ => CertPresence::Unknown,
        }
    };
    let pc_sc = &scan.readers.readers;
    let mut tokens = Vec::new();
    for device in &scan.usb.devices {
        let hint = hints.and_then(|db| db.by_usb(&device.id()));
        if !is_token(device, hint, pc_sc) {
            continue;
        }
        let source = PossibleSource::Usb {
            vid_pid: device.id(),
            product: device.product.clone(),
        };
        tokens.push(TokenFact {
            vid_pid: device.id(),
            hint: hint.map(hint_fact),
            certificates: presence(&source, keys_on_usb(device, hint, keys)),
        });
    }
    let readers = pc_sc
        .iter()
        .filter(|reader| !is_token_reader(reader, &scan.usb.devices, hints))
        .map(|reader| ReaderFact {
            name: reader.name.clone(),
            card: (reader.card == CardState::Present).then(|| {
                let source = PossibleSource::Card {
                    reader: reader.name.clone(),
                    atr: reader.atr.clone(),
                };
                CardFact {
                    atr: reader.atr.clone(),
                    hint: reader
                        .atr
                        .as_deref()
                        .and_then(|atr| hints?.by_atr(atr))
                        .map(hint_fact),
                    certificates: presence(&source, keys_in_reader(&reader.name, keys)),
                }
            }),
        })
        .collect();
    DevicesFact {
        pcscd_running: pcscd_running(scan),
        tokens,
        readers,
    }
}

/// A USB device listed under "Tokens and cards": a known token or card, or
/// an unknown smart card device that is not one of the PC/SC readers.
fn is_token(device: &UsbDevice, hint: Option<&DeviceHint>, readers: &[Reader]) -> bool {
    match hint {
        Some(hint) => hint.kind != DeviceKind::Reader,
        None => device.smart_card && !named_in(device, readers.iter().map(|r| r.name.as_str())),
    }
}

/// A PC/SC reader that is really a token's own chip (a CCID token shows up
/// as a reader named after its USB product).
fn is_token_reader(reader: &Reader, usb: &[UsbDevice], hints: Option<&DeviceDatabase>) -> bool {
    let token_chip = reader
        .atr
        .as_deref()
        .and_then(|atr| hints?.by_atr(atr))
        .is_some_and(|hint| hint.kind == DeviceKind::Token);
    let token_product = usb.iter().any(|device| {
        let hint = hints.and_then(|db| db.by_usb(&device.id()));
        hint.is_some_and(|hint| hint.kind == DeviceKind::Token)
            && named_in(device, std::iter::once(reader.name.as_str()))
    });
    token_chip || token_product
}

fn named_in<'a>(device: &UsbDevice, names: impl IntoIterator<Item = &'a str>) -> bool {
    let product = device.product.as_deref().unwrap_or_default().trim();
    !product.is_empty()
        && names
            .into_iter()
            .any(|name| name.to_lowercase().contains(&product.to_lowercase()))
}

fn keys_on_usb(device: &UsbDevice, hint: Option<&DeviceHint>, keys: &[&FoundKey]) -> u32 {
    let product = device.product.as_deref().unwrap_or_default().trim();
    count(keys, |link| match link {
        DeviceLink::Reader { name } => named_in(device, std::iter::once(name.as_str())),
        DeviceLink::Pkcs11Token { model, .. } => {
            let model = model.trim();
            !model.is_empty()
                && (model.eq_ignore_ascii_case(product)
                    || hint.is_some_and(|h| h.name.to_lowercase().contains(&model.to_lowercase())))
        }
        DeviceLink::CryptoTokenKit { .. } => false,
    })
}

fn keys_in_reader(reader: &str, keys: &[&FoundKey]) -> u32 {
    count(
        keys,
        |link| matches!(link, DeviceLink::Reader { name } if anonymous_reader_name(name) == reader),
    )
}

fn count(keys: &[&FoundKey], on_device: impl Fn(&DeviceLink) -> bool) -> u32 {
    let linked = keys
        .iter()
        .filter(|key| key.device.as_ref().is_some_and(&on_device))
        .count();
    u32::try_from(linked).unwrap_or(u32::MAX)
}

/// What the listed keys say about their devices.
fn linked_devices(keys: &[&FoundKey]) -> LinkedDevices {
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

fn hint_fact(hint: &DeviceHint) -> HintFact {
    HintFact {
        id: hint.id.clone(),
        name: hint.name.clone(),
        driver: hint.driver.as_ref().map(|driver| driver.name.clone()),
        download: hint
            .driver
            .as_ref()
            .and_then(|driver| this_os(&driver.download).cloned()),
    }
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

/// Linux: whether `pcscd` answered. `websign_devices::pcsc::ReaderScan`
/// only says so in its `problem` text (from `Error::NoService` /
/// `Error::ServiceStopped`), so this matches that text. TODO(gustavo): a
/// structured `ReaderScan::service` (running / stopped / unavailable) in
/// `websign-devices` would replace the match.
fn pcscd_running(scan: &Snapshot) -> Option<bool> {
    cfg!(target_os = "linux").then(|| {
        scan.readers
            .problem
            .as_deref()
            .is_none_or(|problem| !problem.contains("not running"))
    })
}
