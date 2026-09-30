//! Live updates of the Devices tab (`docs/ux.md` §8.4: "The list updates
//! live"): PC/SC reader and card events wake the window, which scans again.

use std::sync::mpsc::{Receiver, channel};

use websign_devices::monitor::{self, DeviceEvent, MonitorHandle};

/// A running device monitor.
#[derive(Debug)]
pub struct Live {
    _monitor: MonitorHandle,
    changes: Receiver<()>,
}

impl Live {
    /// Starts watching; `wake` is called on every event (from another
    /// thread) so the window runs a frame to notice it.
    pub fn start(wake: impl Fn() + Send + 'static) -> Live {
        let (events, received) = channel::<DeviceEvent>();
        let (changed, changes) = channel();
        let monitor = monitor::start(events);
        let forwarder = std::thread::Builder::new()
            .name("diagnostics-live".to_owned())
            .spawn(move || {
                for _ in received {
                    if changed.send(()).is_err() {
                        return;
                    }
                    wake();
                }
            });
        if let Err(error) = forwarder {
            log::warn!("diagnostics: live device updates unavailable: {error}");
        }
        Live {
            _monitor: monitor,
            changes,
        }
    }

    /// Whether anything changed since the last call.
    pub fn changed(&self) -> bool {
        self.changes.try_iter().count() > 0
    }
}
