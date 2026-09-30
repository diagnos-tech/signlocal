//! `websign diagnostics` (and `websign` with no command): the diagnostics
//! window, in this process.

use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::ValueEnum;
use websign_protocol::messages::DiagnosticsTab;

use crate::ui::renderer;

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

impl From<TabArg> for DiagnosticsTab {
    fn from(tab: TabArg) -> DiagnosticsTab {
        match tab {
            TabArg::Browsers => DiagnosticsTab::Browsers,
            TabArg::Devices => DiagnosticsTab::Devices,
            TabArg::Certificates => DiagnosticsTab::Certificates,
            TabArg::Help => DiagnosticsTab::Help,
        }
    }
}

/// Registers silently (every start repairs manifests), then opens the window.
pub fn run(args: &DiagnosticsArgs) -> ExitCode {
    super::install::repair_silently();
    open(args.tab.map(DiagnosticsTab::from))
}

/// Set while this process shows the diagnostics window.
static OPEN: AtomicBool = AtomicBool::new(false);

/// Whether this process shows the diagnostics window (a `websign:` URL
/// event then needs no second one).
pub fn is_open() -> bool {
    OPEN.load(Ordering::Relaxed)
}

/// Opens the window on this (the main) thread until it is closed. Nothing
/// has read stdin here, so a renderer failure may re-execute with glow
/// (after remembering it for later launches, `ui::renderer`).
pub fn open(tab: Option<DiagnosticsTab>) -> ExitCode {
    OPEN.store(true, Ordering::Relaxed);
    let result = crate::ui::diagnostics::run(tab);
    OPEN.store(false, Ordering::Relaxed);
    let error = match result {
        Ok(()) => return ExitCode::SUCCESS,
        Err(error) => error,
    };
    log::error!("the diagnostics window could not open: {error}");
    if renderer::is_renderer_failure(&error) && renderer::may_fall_back_to_glow() {
        renderer::remember_glow_needed();
        if let Some(code) = rerun_with_glow() {
            return code;
        }
    }
    eprintln!("websign: the diagnostics window could not open: {error}");
    ExitCode::from(super::output::FAILURE)
}

/// This command again under glow; its exit code, or `None` if it could not
/// be started.
fn rerun_with_glow() -> Option<ExitCode> {
    let status = renderer::glow_reexec().and_then(|mut command| command.status());
    match status {
        Ok(status) => Some(crate::host_process::exit_code(status.code().unwrap_or(1))),
        Err(error) => {
            log::error!("glow re-exec failed: {:?}", error.kind());
            None
        }
    }
}
