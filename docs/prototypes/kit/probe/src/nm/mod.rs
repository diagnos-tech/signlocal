//! Native messaging: running as the browser's host and registering with browsers.
//!
//! Layout: [`launch`] recognizes a browser start, [`framing`] and
//! [`protocol`] speak the wire format, [`handler`] applies the protocol's
//! rules on top of a [`backend::Backend`], and [`host`] runs the loop.
//! [`register`] writes the manifests that make browsers find all this.

mod backend;
mod base64;
mod framing;
mod handler;
pub mod host;
mod keystore_backend;
mod launch;
mod log;
mod protocol;
pub mod register;
mod summary;

use std::process::ExitCode;

pub use launch::{BrowserLaunch, detect_browser_launch, is_chromium_extension_id};

/// Serves native messaging on stdin/stdout until the browser disconnects.
pub fn run_host(launch: BrowserLaunch) -> ExitCode {
    host::serve_stdio(&launch)
}
