//! Hardware that might hold a certificate: USB tokens and smart card readers,
//! live reader/card events, and the hints of `devices.json`.
//!
//! Only descriptive data (VID:PID, reader names, ATR); nothing here talks to
//! a card beyond PC/SC status. USB serial numbers are never read and are cut
//! out of reader names, because diagnostics built from this data end up in
//! public issues.
//!
//! [`usb`] and [`pcsc`] are promoted from the Phase-0 kit
//! (`probe/src/devices`), reviewed; [`monitor`], [`hints`] and [`possible`]
//! are new (`SPEC.md`).

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod hints;
pub mod monitor;
pub mod pcsc;
pub mod possible;
pub mod usb;

pub use pcsc::anonymous_reader_name;

use serde::Serialize;

/// Everything found in one pass, and what could not be looked at (no PC/SC
/// service, no permission to enumerate USB): a machine without hardware is
/// not an error.
#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub usb: usb::UsbScan,
    pub readers: pcsc::ReaderScan,
}

impl Snapshot {
    /// Scans USB (smart card class and devices known to `devices.json`) and
    /// PC/SC readers.
    pub fn scan() -> Snapshot {
        todo!("SPEC.md §1")
    }
}
