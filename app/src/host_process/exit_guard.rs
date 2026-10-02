//! The last word on "nothing outlives the connection": once the engine has
//! ended, the window gets a short grace to finish a notice's hold and close;
//! if its event loop is still running after that (an OS that stopped
//! delivering events to a hidden window, a stuck modal viewer), the process
//! exits anyway with the engine's status.

use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

/// Longer than any notice's hold (1.5 s) plus the closing frames.
const GRACE: Duration = Duration::from_secs(5);

/// The UI thread's half: dropping it says the event loop has returned.
#[derive(Debug)]
pub struct UiReturned {
    _alive: Sender<()>,
}

/// The engine thread's half.
#[derive(Debug)]
pub struct ExitGuard(Receiver<()>);

/// A connected pair.
pub fn pair() -> (UiReturned, ExitGuard) {
    let (sender, receiver) = channel();
    (UiReturned { _alive: sender }, ExitGuard(receiver))
}

impl ExitGuard {
    /// Called by the engine thread once the engine ended with `status`:
    /// returns when the UI thread has, or ends the process after [`GRACE`].
    pub fn wait_for_ui(self, status: i32) {
        if !self.ui_returned_within(GRACE) {
            log::error!("the window did not close after the connection ended; exiting");
            std::process::exit(status);
        }
    }

    fn ui_returned_within(&self, grace: Duration) -> bool {
        !matches!(self.0.recv_timeout(grace), Err(RecvTimeoutError::Timeout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_once_the_ui_has_and_notices_when_it_has_not() {
        let (ui, guard) = pair();
        assert!(!guard.ui_returned_within(Duration::from_millis(10)));
        drop(ui);
        assert!(guard.ui_returned_within(Duration::from_secs(5)));
    }
}
