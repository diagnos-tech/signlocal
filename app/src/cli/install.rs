//! `websign install` / `websign uninstall`: what an installer runs once, and
//! what the app repeats silently on every start (`docs/architecture/desktop-api.md`).

use std::process::ExitCode;

use super::options::BrowserArg;

/// `websign install`.
#[derive(Debug, clap::Args)]
pub struct InstallArgs {
    /// Browsers to register with (repeatable). Default: all.
    #[arg(long = "browser", value_enum, value_name = "BROWSER")]
    pub browsers: Vec<BrowserArg>,
    /// Linux packages only (run as root): system-wide manifests, which the
    /// Firefox Snap portal reads, instead of per-user ones.
    #[arg(long)]
    pub system: bool,
    /// Print what would be done; change nothing.
    #[arg(long)]
    pub dry_run: bool,
    /// Print a JSON report instead of text.
    #[arg(long)]
    pub json: bool,
}

/// `websign uninstall`.
#[derive(Debug, clap::Args)]
pub struct UninstallArgs {
    /// Also delete settings, remembered sites and logs.
    #[arg(long)]
    pub purge: bool,
    #[arg(long)]
    pub system: bool,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
}

/// Registers manifests, the URL scheme, extension pre-registration (Windows)
/// and the app-menu entry (Linux `.desktop`).
pub fn run_install(args: &InstallArgs) -> ExitCode {
    let _ = args;
    todo!("desktop-api.md §install")
}

/// Removes everything `install` wrote.
pub fn run_uninstall(args: &UninstallArgs) -> ExitCode {
    let _ = args;
    todo!("desktop-api.md §uninstall")
}
