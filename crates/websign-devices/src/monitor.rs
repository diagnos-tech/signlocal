//! Live reader and card events, so the certificate list updates by itself.
//!
//! A background thread blocks in `SCardGetStatusChange` over every reader
//! plus the special `\\?PnP?\Notification` reader (reader plugged in or
//! removed). Each change is sent as a [`DeviceEvent`]; the host invalidates
//! the key store cache and re-lists. USB tokens without PC/SC (rare; some
//! HID tokens) are caught by the periodic re-scan the host does while a
//! window is open.

mod session;
mod tracker;

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use pcsc::Context;

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
    stop: Option<Sender<()>>,
    slot: Arc<ContextSlot>,
    thread: Option<JoinHandle<()>>,
}

/// The PC/SC context the thread is currently blocked in, so dropping the
/// handle can interrupt the wait instead of waiting for its timeout.
#[derive(Default)]
pub(crate) struct ContextSlot(Mutex<Option<Context>>);

impl std::fmt::Debug for ContextSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContextSlot")
    }
}

impl ContextSlot {
    pub(crate) fn set(&self, context: Option<Context>) {
        if let Ok(mut slot) = self.0.lock() {
            *slot = context;
        }
    }

    fn cancel(&self) {
        if let Ok(slot) = self.0.lock()
            && let Some(context) = slot.as_ref()
        {
            let _ = context.cancel();
        }
    }
}

/// Starts watching. Events go to `events` until the handle is dropped or the
/// receiver hangs up. Never fails: without a PC/SC service the thread retries
/// every few seconds and reports [`DeviceEvent::ServiceChanged`] when it
/// appears.
pub fn start(events: Sender<DeviceEvent>) -> MonitorHandle {
    let (stop, stopped) = mpsc::channel();
    let slot = Arc::new(ContextSlot::default());
    let worker_slot = Arc::clone(&slot);
    let thread = thread::Builder::new()
        .name("device-monitor".to_owned())
        .spawn(move || session::run(events, stopped, &worker_slot))
        .inspect_err(|error| log::warn!("device monitor could not start: {error}"))
        .ok();
    MonitorHandle {
        stop: Some(stop),
        slot,
        thread,
    }
}

impl Drop for MonitorHandle {
    fn drop(&mut self) {
        // Hang up first so the thread sees the request even if the cancel
        // lands between two of its PC/SC calls.
        self.stop.take();
        self.slot.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
