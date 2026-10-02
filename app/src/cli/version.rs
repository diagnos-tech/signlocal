//! `websign version`.

use std::process::ExitCode;

use websign_protocol::types::AppInfo;

use super::output::{FAILURE, SUCCESS, print_json};

/// `websign version`.
#[derive(Debug, clap::Args)]
pub struct VersionArgs {
    /// Print `AppInfo` as JSON (version, protocols, os, arch, channel).
    #[arg(long)]
    pub json: bool,
}

/// Prints `websign 1.4.0 (protocol 1)` or the JSON.
pub fn run(args: &VersionArgs) -> ExitCode {
    let info = crate::host_process::app_info();
    match render(&info, args.json) {
        Some(text) => {
            print_json(&text);
            ExitCode::from(SUCCESS)
        }
        None => ExitCode::from(FAILURE),
    }
}

/// The text or JSON for `info`.
pub fn render(info: &AppInfo, json: bool) -> Option<String> {
    if json {
        serde_json::to_string(info).ok()
    } else {
        Some(format!(
            "{} {} (protocol {})",
            websign_project::SLUG,
            info.version,
            info.protocols.max
        ))
    }
}
