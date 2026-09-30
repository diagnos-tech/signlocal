//! `devices.json`: which driver a token or card needs, per OS.
//!
//! Hints only help, they never block (`docs/ux.md` R7): a device without an
//! entry still works when its driver is installed. The file is embedded at
//! build time and validated against `devices.schema.json` in CI.

use serde::Deserialize;

/// The embedded `devices.json` (repository root, CC0-1.0).
pub const DEVICES_JSON: &str = include_str!("../../../devices.json");

/// The parsed database.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeviceDatabase {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub version: u32,
    pub devices: Vec<DeviceHint>,
}

/// One known model (`docs/ux.md` R7).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeviceHint {
    /// Stable kebab-case id: `"safenet-etoken-5110"`.
    pub id: String,
    /// Commercial name shown to people.
    pub name: String,
    pub kind: DeviceKind,
    #[serde(rename = "match")]
    pub matches: DeviceMatch,
    pub driver: Option<DriverHint>,
    pub macos: Option<MacosHint>,
    pub pin_unlock: Option<PinUnlockHint>,
}

/// What the device is, for the words the UI uses ("token", "card").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceKind {
    Token,
    Card,
    Reader,
}

/// How a device is recognized.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceMatch {
    /// `"0529:0620"` (lowercase hex VID:PID).
    #[serde(default)]
    pub usb: Vec<String>,
    /// ATR patterns: uppercase hex, `..` matches any byte
    /// (`"3BD518FF8191FE1FC38073C8211..."`).
    #[serde(default)]
    pub atr: Vec<String>,
}

/// The vendor middleware.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverHint {
    pub name: String,
    #[serde(default)]
    pub download: PerOs<String>,
    #[serde(default)]
    pub pkcs11: PerOs<String>,
}

/// A value per OS; any may be missing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerOs<T> {
    pub windows: Option<T>,
    pub macos: Option<T>,
    pub linux: Option<T>,
}

/// macOS facts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacosHint {
    /// `false` → the store build needs the Add-on for this device (`docs/ux.md` §7).
    pub cryptotokenkit: bool,
}

/// Where a locked PIN is unlocked with the PUK.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinUnlockHint {
    pub tool: String,
}

/// Why `devices.json` could not be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("devices.json: {0}")]
pub struct HintsError(pub String);

impl DeviceDatabase {
    /// Parses the embedded file.
    pub fn embedded() -> Result<DeviceDatabase, HintsError> {
        todo!("SPEC.md §3")
    }

    /// The entry matching a USB VID:PID (`"0529:0620"`), if any.
    pub fn by_usb(&self, vid_pid: &str) -> Option<&DeviceHint> {
        let _ = vid_pid;
        todo!("SPEC.md §3")
    }

    /// The first entry whose ATR pattern matches `atr` (uppercase hex).
    pub fn by_atr(&self, atr: &str) -> Option<&DeviceHint> {
        let _ = atr;
        todo!("SPEC.md §3")
    }
}
