//! "Possible certificates": devices that look like a token or card but
//! brought no certificate to the list (`docs/ux.md` §6.1). Pure logic.

use crate::Snapshot;
use crate::hints::{DeviceDatabase, DeviceHint};

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
pub fn possible_devices(
    snapshot: &Snapshot,
    db: &DeviceDatabase,
    linked: &LinkedDevices,
) -> Vec<PossibleDevice> {
    let _ = (snapshot, db, linked);
    todo!("SPEC.md §4")
}
