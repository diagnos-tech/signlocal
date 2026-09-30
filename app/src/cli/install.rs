//! `websign install` / `websign uninstall`: what an installer runs once, and
//! what the app repeats silently on every start (`docs/architecture/desktop-api.md`).
//!
//! Registration is files and registry keys only: nothing is started at
//! login, no service is installed, the app still runs only when called.

mod menu_entry;
mod purge;
mod steps;

use std::process::ExitCode;

use websign_registration::{Action, Outcome};

use super::options::BrowserArg;
use super::report::Report;

pub use steps::manifests as manifest_steps;

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
    /// Remove the system-wide manifests (Linux packages, as root).
    #[arg(long)]
    pub system: bool,
    /// Print what would be done; change nothing.
    #[arg(long)]
    pub dry_run: bool,
    /// Print a JSON report instead of text.
    #[arg(long)]
    pub json: bool,
}

/// Registers manifests, the URL scheme, extension pre-registration (Windows)
/// and the app-menu entry (Linux `.desktop`). `--system` writes only the
/// system manifests: the rest is per user.
pub fn run_install(args: &InstallArgs) -> ExitCode {
    let mut request = steps::request(Action::Install, args.system, args.dry_run);
    request.browsers = args.browsers.iter().map(|&b| b.into()).collect();
    let mut report = Report {
        command: "install",
        system: args.system,
        dry_run: args.dry_run,
        steps: steps::manifests(&request),
    };
    if !args.system {
        report
            .steps
            .push(steps::url_scheme(Action::Install, args.dry_run));
        report
            .steps
            .extend(steps::preregistration(Action::Install, args.dry_run));
        report.steps.extend(menu_entry::apply(true, args.dry_run));
    }
    log_summary(&report);
    report.print(args.json)
}

/// Removes everything `install` wrote (and with `--purge`, the app's data).
pub fn run_uninstall(args: &UninstallArgs) -> ExitCode {
    let request = steps::request(Action::Uninstall, args.system, args.dry_run);
    let mut report = Report {
        command: "uninstall",
        system: args.system,
        dry_run: args.dry_run,
        steps: steps::manifests(&request),
    };
    if !args.system {
        report
            .steps
            .push(steps::url_scheme(Action::Uninstall, args.dry_run));
        report
            .steps
            .extend(steps::preregistration(Action::Uninstall, args.dry_run));
        report.steps.extend(menu_entry::apply(false, args.dry_run));
    }
    if args.purge {
        report.steps.extend(purge::purge(args.dry_run));
    }
    log_summary(&report);
    report.print(args.json)
}

/// Rewrites this user's manifests without output: every diagnostics start
/// does it, so a browser installed later finds the app (and a moved binary
/// is pointed at again).
pub fn repair_silently() {
    log_summary(&silent_report(false));
}

/// What `websign:activate` does for store builds, which run no installer:
/// the manifests and the extension pre-registration, silently.
pub fn activate_silently() {
    log_summary(&silent_report(true));
}

fn silent_report(preregister: bool) -> Report {
    let mut steps = steps::manifests(&steps::request(Action::Install, false, false));
    if preregister {
        steps.extend(steps::preregistration(Action::Install, false));
    }
    Report {
        command: if preregister { "activate" } else { "repair" },
        system: false,
        dry_run: false,
        steps,
    }
}

/// Counts only: step targets and locations name paths with the login.
fn log_summary(report: &Report) {
    let count = |f: fn(&Outcome) -> bool| report.steps.iter().filter(|s| f(&s.outcome)).count();
    log::info!(
        "{}: {} written, {} removed, {} skipped, {} failed",
        report.command,
        count(|o| matches!(o, Outcome::Written)),
        count(|o| matches!(o, Outcome::Removed)),
        count(|o| matches!(o, Outcome::Skipped(_) | Outcome::NotPresent)),
        count(|o| matches!(o, Outcome::Failed(_))),
    );
}
