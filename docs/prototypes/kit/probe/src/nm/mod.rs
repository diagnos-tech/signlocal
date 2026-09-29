//! Native messaging: running as the browser's host and registering with browsers.

pub mod host;
pub mod register;

use std::process::ExitCode;

/// How a browser started this process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserLaunch {
    /// The calling extension's origin as the browser reported it.
    pub origin: String,
}

/// Recognizes the arguments browsers pass when they start a native host.
pub fn detect_browser_launch() -> Option<BrowserLaunch> {
    None
}

/// Serves native messaging on stdin/stdout until the browser disconnects.
pub fn run_host(launch: BrowserLaunch) -> ExitCode {
    let _ = launch;
    todo!("native messaging host")
}
