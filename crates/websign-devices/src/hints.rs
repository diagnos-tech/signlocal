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
        serde_json::from_str(DEVICES_JSON).map_err(|error| HintsError(error.to_string()))
    }

    /// The entry matching a USB VID:PID (`"0529:0620"`), if any.
    pub fn by_usb(&self, vid_pid: &str) -> Option<&DeviceHint> {
        let vid_pid = vid_pid.trim();
        self.devices.iter().find(|device| {
            device
                .matches
                .usb
                .iter()
                .any(|known| known.eq_ignore_ascii_case(vid_pid))
        })
    }

    /// The first entry whose ATR pattern matches `atr` (uppercase hex).
    pub fn by_atr(&self, atr: &str) -> Option<&DeviceHint> {
        self.devices.iter().find(|device| {
            device
                .matches
                .atr
                .iter()
                .any(|pattern| crate::atr::matches(pattern, atr))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> DeviceDatabase {
        DeviceDatabase::embedded().expect("the embedded devices.json parses")
    }

    #[test]
    fn the_embedded_file_parses_and_has_unique_ids() {
        let db = database();
        assert_eq!(db.version, 1);
        let mut ids: Vec<_> = db.devices.iter().map(|device| device.id.as_str()).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total);
    }

    #[test]
    fn usb_ids_match_exactly_and_ignore_case() {
        let db = database();
        let hint = db.by_usb("0529:0620").expect("eToken 5110 is known");
        assert_eq!(hint.id, "safenet-etoken-5110");
        assert_eq!(db.by_usb("0529:0620"), db.by_usb(" 0529:0620 "));
        assert_eq!(
            db.by_usb("163C:0407").map(|d| d.id.as_str()),
            Some("watchdata-proxkey")
        );
        assert!(db.by_usb("0529:062").is_none());
        assert!(db.by_usb("dead:beef").is_none());
        assert!(db.by_usb("").is_none());
    }

    #[test]
    fn atr_patterns_match_with_wildcards_and_first_entry_wins() {
        let db = database();
        let card: &str = "3B7D95000080318065B08311AABB83009000";
        assert_eq!(
            db.by_atr(card).map(|d| d.id.as_str()),
            Some("pt-cartao-de-cidadao")
        );
        let exact = "3BFF9600008131FE4380318065B0855956FB120FFE82900000";
        assert_eq!(
            db.by_atr(exact).map(|d| d.id.as_str()),
            Some("safenet-etoken-5110")
        );
        assert!(db.by_atr("3B00").is_none());
        assert!(db.by_atr("").is_none());
    }

    #[test]
    fn every_entry_is_well_formed() {
        for device in database().devices {
            for usb in &device.matches.usb {
                assert_eq!(usb, &usb.to_ascii_lowercase(), "{}", device.id);
            }
            for atr in &device.matches.atr {
                assert!(atr.len().is_multiple_of(2), "{}: odd ATR length", device.id);
            }
        }
    }
}
