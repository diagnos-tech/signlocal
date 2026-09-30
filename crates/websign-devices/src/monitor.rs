//! Live reader and card events, so the certificate list updates by itself.
//!
//! A background thread blocks in `SCardGetStatusChange` over every reader
//! plus the special `\\?PnP?\Notification` reader (reader plugged in or
//! removed). Each change is sent as a [`DeviceEvent`]; the host invalidates
//! the key store cache and re-lists. USB tokens without PC/SC (rare; some
//! HID tokens) are caught by the periodic re-scan the host does while a
//! window is open.

use std::sync::mpsc::Sender;

/// Something changed that may change the certificate list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceEvent {
    ReaderAdded {
        reader: String,
    },
    ReaderRemoved {
        reader: String,
    },
    CardInserted {
        reader: String,
        atr: Option<String>,
    },
    CardRemoved {
        reader: String,
    },
    /// The PC/SC service stopped or started (pcscd socket activation).
    ServiceChanged {
        running: bool,
    },
}

/// Stops the monitor thread when dropped.
#[derive(Debug)]
pub struct MonitorHandle {
    _private: (),
}

/// Starts watching. Events go to `events` until the handle is dropped or the
/// receiver hangs up. Never fails: without a PC/SC service the thread retries
/// every few seconds and reports [`DeviceEvent::ServiceChanged`] when it
/// appears.
pub fn start(events: Sender<DeviceEvent>) -> MonitorHandle {
    let _ = events;
    todo!("SPEC.md §2")
}
