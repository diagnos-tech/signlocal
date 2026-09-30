//! `websign`: the WebeSign desktop app.
//!
//! One binary, started four ways, told apart before any argument parsing:
//!
//! 1. by a **browser** for our extension (native messaging: the arguments
//!    are the extension origin or a manifest path) → host on stdio;
//! 2. by the OS for a **`websign:` URL** (`websign:activate`) → register with
//!    browsers and open diagnostics;
//! 3. by a **program** or a person on the command line → [`cli`];
//! 4. with **no arguments** (Start menu, Launchpad, app menu) → diagnostics.
//!
//! Nothing runs in the background: every start ends when its job ends.

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
    match launch::detect() {
        launch::Launch::Browser(browser) => host_process::serve_browser(browser),
        launch::Launch::Url(url) => cli::activate::run(&url),
        launch::Launch::Command => cli::run(),
    }
}
