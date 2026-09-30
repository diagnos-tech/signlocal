//! `websign:activate` from the website's `/activate` page: register with
//! every browser (stores run no install scripts), then open diagnostics with
//! "Getting started". Any other `websign:` URL only opens diagnostics.

use std::process::ExitCode;

/// Handles the URL the OS passed.
pub fn run(url: &str) -> ExitCode {
    let _ = url;
    todo!("desktop-api.md §URL scheme")
}
