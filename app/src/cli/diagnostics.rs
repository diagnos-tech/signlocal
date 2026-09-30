//! `websign diagnostics` (and `websign` with no command): the diagnostics
//! window, in this process.

use std::process::ExitCode;

use clap::ValueEnum;

/// `websign diagnostics`.
#[derive(Debug, Default, clap::Args)]
pub struct DiagnosticsArgs {
    /// Open this tab instead of the first one with a problem.
    #[arg(long, value_enum)]
    pub tab: Option<TabArg>,
}

/// `--tab`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TabArg {
    Browsers,
    Devices,
    Certificates,
    Help,
}

/// Registers silently (every start repairs manifests), then opens the window.
pub fn run(args: &DiagnosticsArgs) -> ExitCode {
    let _ = args;
    todo!("desktop-api.md §diagnostics; ui/diagnostics track")
}
