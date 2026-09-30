//! The diagnostics window (`docs/ux.md` §8), 760 × 540: Browsers, Devices,
//! Certificates, Help.

/// The eframe app state of the diagnostics window.
#[derive(Debug)]
pub struct DiagnosticsWindow {
    _private: (),
}

/// Runs the diagnostics window on the calling (main) thread until it is
/// closed, on `tab` or else the first tab with a problem. `Err` carries the
/// reason for the log when no window could be opened.
pub fn run(tab: Option<websign_protocol::messages::DiagnosticsTab>) -> Result<(), String> {
    let _ = tab;
    todo!("ui/diagnostics track: the window")
}
