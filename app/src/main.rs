//! `websign`: the SignLocal desktop app.
//!
//! One binary, started four ways, told apart before any argument parsing
//! ([`launch`]):
//!
//! 1. by a **browser** for our extension (native messaging: the arguments
//!    are the extension origin or a manifest path) → host on stdio;
//! 2. by the OS for a **`websign:` URL** (`websign:activate`) → register with
//!    browsers and open diagnostics (on macOS the URL arrives as an Apple
//!    Event instead, [`platform::url_events`]);
//! 3. by a **program** or a person on the command line → [`cli`]
//!    (`websign connect` is the desktop API on stdio);
//! 4. with **no arguments** (Start menu, Launchpad, app menu) → diagnostics.
//!
//! Nothing runs in the background and nothing is installed to start later:
//! every start ends when its job ends.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
// Declared modules are not all reachable from `main` yet; remove this once
// every module is wired (CI then enforces dead-code warnings again).
#![allow(dead_code)]

mod cli;
mod host_process;
mod launch;
mod logging;
mod platform;
mod ui;

#[cfg(feature = "e2e")]
mod e2e;

use std::process::ExitCode;

fn main() -> ExitCode {
    logging::init();
    // macOS delivers `websign:` URLs as events, to a new or a running copy.
    platform::url_events::listen(cli::activate::on_url_event);
    match launch::detect() {
        launch::Launch::Browser(browser) => host_process::serve_browser(browser),
        launch::Launch::Url(action) => cli::activate::run(action),
        launch::Launch::Connect | launch::Launch::Gui | launch::Launch::Command => cli::run(),
    }
}
