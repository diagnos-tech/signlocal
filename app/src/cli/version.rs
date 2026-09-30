//! `websign version`.

use std::process::ExitCode;

/// `websign version`.
#[derive(Debug, clap::Args)]
pub struct VersionArgs {
    /// Print `AppInfo` as JSON (version, protocols, os, arch, channel).
    #[arg(long)]
    pub json: bool,
}

/// Prints `websign 1.4.0 (protocol 1)` or the JSON.
pub fn run(args: &VersionArgs) -> ExitCode {
    let _ = args;
    todo!("desktop-api.md §version")
}
