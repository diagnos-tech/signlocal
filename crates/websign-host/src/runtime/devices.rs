//! The reader and card monitor, feeding the engine.

use std::sync::mpsc::channel;

use websign_devices::monitor::{self, MonitorHandle};

use super::EventSender;
use crate::engine::EngineEvent;

/// Starts watching readers and cards; the events become
/// [`EngineEvent::Device`]. The returned handle stops the monitor when
/// dropped, so the caller keeps it for the life of the process. The monitor
/// never fails to start: without PC/SC it retries on its own thread, and if
/// that thread dies the list simply stops updating by itself ("Scan again"
/// still works).
pub(super) fn start_monitor(events: EventSender) -> MonitorHandle {
    let (sender, receiver) = channel();
    let handle = monitor::start(sender);
    std::thread::spawn(move || {
        for event in receiver {
            if events.send(EngineEvent::Device(event)).is_err() {
                return;
            }
        }
    });
    handle
}
