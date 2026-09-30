//! A host process: the engine on a worker thread, the UI on the main thread.
//!
//! The main thread must own the UI event loop (macOS requires it), and winit
//! cannot create a second event loop in one process, so the event loop starts
//! lazily with the first window and lives until the process exits; the
//! window is hidden between requests.

use std::process::ExitCode;

use websign_core::present::caller::DesktopCaller;
use websign_host::BrowserLaunch;

/// Serves a browser's native messaging port.
pub fn serve_browser(launch: BrowserLaunch) -> ExitCode {
    let _ = launch;
    todo!("docs/architecture/overview.md §Processes; host_process track")
}

/// Serves `websign connect` for the program `caller`.
pub fn serve_desktop(caller: DesktopCaller) -> ExitCode {
    let _ = caller;
    todo!("docs/architecture/overview.md §Processes; host_process track")
}
