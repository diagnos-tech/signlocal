//! "Possible certificates": devices that look like a token or card but
//! brought no certificate to the list (`docs/ux.md` §6.1). Pure logic.

use crate::Snapshot;
use crate::hints::{DeviceDatabase, DeviceHint, DeviceKind};
use crate::pcsc::{CardState, Reader, anonymous_reader_name};
use crate::usb::UsbDevice;

/// A device worth a hint.
#[derive(Debug, Clone, PartialEq)]
pub struct PossibleDevice {
    pub source: PossibleSource,
    /// The `devices.json` entry, when known.
    pub hint: Option<DeviceHint>,
    /// Whether the device may be shown in the confirmation window. False
    /// when the certificate ↔ device mapping is uncertain: the hint then
    /// appears only in diagnostics ("We can't tell if it has certificates").
    pub confident: bool,
}

/// Where the device was seen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PossibleSource {
    /// USB VID:PID (`"0529:0620"`).
    Usb {
        vid_pid: String,
        product: Option<String>,
    },
    /// A reader with a card; `atr` uppercase hex.
    Card { reader: String, atr: Option<String> },
}

/// What is known about the listed keys' devices (reader names, token models)
/// to take devices that already brought certificates out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkedDevices {
    pub readers: Vec<String>,
    pub token_models: Vec<String>,
}

/// Devices from `snapshot` that are known to `db` or are CCID class, minus
/// those linked to a listed certificate.
///
/// USB devices come first (by VID:PID), then readers holding a card (by
/// name). A bare reader is never "confident": it may hold no card at all.
pub fn possible_devices(
    snapshot: &Snapshot,
    db: &DeviceDatabase,
    linked: &LinkedDevices,
) -> Vec<PossibleDevice> {
    let mut found: Vec<PossibleDevice> = snapshot
        .usb
        .devices
        .iter()
        .filter_map(|device| usb_candidate(device, db, linked))
        .collect();
    found.sort_by(|a, b| a.source.sort_key().cmp(b.source.sort_key()));

    let mut cards: Vec<PossibleDevice> = snapshot
        .readers
        .readers
        .iter()
        .filter_map(|reader| card_candidate(reader, db, linked))
        .collect();
    cards.sort_by(|a, b| a.source.sort_key().cmp(b.source.sort_key()));

    found.extend(cards);
    found
}

impl PossibleSource {
    fn sort_key(&self) -> &str {
        match self {
            PossibleSource::Usb { vid_pid, .. } => vid_pid,
            PossibleSource::Card { reader, .. } => reader,
        }
    }
}

fn usb_candidate(
    device: &UsbDevice,
    db: &DeviceDatabase,
    linked: &LinkedDevices,
) -> Option<PossibleDevice> {
    let vid_pid = device.id();
    let hint = db.by_usb(&vid_pid);
    if !device.smart_card && hint.is_none() {
        return None;
    }
    if let Some(hint) = hint
        && linked
            .token_models
            .iter()
            .any(|model| model.trim().eq_ignore_ascii_case(&hint.name))
    {
        return None;
    }
    Some(PossibleDevice {
        source: PossibleSource::Usb {
            vid_pid,
            product: device.product.clone(),
        },
        confident: hint.is_some_and(|hint| hint.kind != DeviceKind::Reader),
        hint: hint.cloned(),
    })
}

fn card_candidate(
    reader: &Reader,
    db: &DeviceDatabase,
    linked: &LinkedDevices,
) -> Option<PossibleDevice> {
    if reader.card != CardState::Present {
        return None;
    }
    let listed = linked
        .readers
        .iter()
        .any(|name| anonymous_reader_name(name) == reader.name);
    if listed {
        return None;
    }
    let hint = reader.atr.as_deref().and_then(|atr| db.by_atr(atr));
    Some(PossibleDevice {
        source: PossibleSource::Card {
            reader: reader.name.clone(),
            atr: reader.atr.clone(),
        },
        confident: hint.is_some(),
        hint: hint.cloned(),
    })
}

#[cfg(test)]
mod tests;
