//! `websign-probe`: lists every signing certificate the machine exposes and
//! signs a random digest with each, through every path the app will use —
//! Windows CNG/CAPI, macOS Keychain/CryptoTokenKit and PKCS#11 — then checks
//! the result. It also doubles as a native messaging host so browser
//! integration can be tested with the very same binary.

mod cli;
mod commands;
mod config;
mod devices;
mod keystores;
mod nm;

use std::process::ExitCode;

fn main() -> ExitCode {
    // Browsers start the host with their own arguments, never ours, so this
    // check comes before any command-line parsing.
    if let Some(launch) = nm::detect_browser_launch() {
        return nm::run_host(launch);
    }
    cli::run()
}
