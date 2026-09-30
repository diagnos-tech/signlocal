//! `websign doctor`: the diagnostics report on stdout, for support and CI.

use std::process::ExitCode;

/// `websign doctor`.
#[derive(Debug, clap::Args)]
pub struct DoctorArgs {
    /// Print the report's data as JSON instead of the text.
    #[arg(long)]
    pub json: bool,
}

/// Prints `websign_ui_model::diagnostics::report::render` (or its input as
/// JSON). Exit 0 even when things are red: the report is the answer.
pub fn run(args: &DoctorArgs) -> ExitCode {
    let _ = args;
    todo!("desktop-api.md §doctor")
}
