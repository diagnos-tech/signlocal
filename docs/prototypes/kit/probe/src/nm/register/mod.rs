//! `websign-probe register`: writes the native messaging host manifest where
//! each browser looks for it.
//!
//! Registration writes only into folders a browser already created. Creating
//! `~/.config/vivaldi` for a machine without Vivaldi would litter the home
//! directory and mislead tools that treat the folder as proof of an install;
//! the price is that a browser installed but never started is skipped until
//! the next registration. The final app registers on every start, so that gap
//! closes by itself. On Windows there is no such folder to test: registry
//! keys are written for every selected browser, since an unused key is
//! invisible.

mod browsers;
mod home;
mod linux;
mod macos;
mod manifest;
mod target;
mod windows;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, bail, ensure};

use browsers::{Browser, BrowserChoice, Family};
use target::{Action, Context as ApplyContext, Outcome, Target, apply};

use super::log::log_path;

/// Arguments of `websign-probe register`.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Browser to register with (repeatable). Default: all of them.
    #[arg(long = "browser", value_enum, value_name = "BROWSER")]
    pub browsers: Vec<BrowserChoice>,
    /// Remove the registration instead of writing it.
    #[arg(long)]
    pub uninstall: bool,
    /// Register only in `<DIR>/NativeMessagingHosts/`, which Chromium reads
    /// when started with `--user-data-dir=<DIR>` on Linux and macOS. For tests
    /// that must not touch the real browser profile. Not usable on Windows,
    /// where hosts are found through the registry.
    #[arg(long, value_name = "DIR")]
    pub user_data_dir: Option<PathBuf>,
    /// Print what would be done and change nothing.
    #[arg(long)]
    pub dry_run: bool,
    /// Also allow this Chromium extension ID (repeatable).
    #[arg(long = "extension-id", value_name = "ID")]
    pub extension_ids: Vec<String>,
    /// Windows only: folder for the manifest files that registry keys point
    /// to (default: %LOCALAPPDATA%\<slug>\NativeMessagingHosts). A packaged
    /// (MSIX) app must pick a folder browsers can read.
    #[arg(long, value_name = "DIR")]
    pub manifest_dir: Option<PathBuf>,
}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let action = if args.uninstall {
        Action::Uninstall
    } else {
        Action::Install
    };
    let browsers = browsers::selected(&args.browsers);
    let targets = plan(args, &browsers)?;
    let host = host_binary()?;
    let origins = manifest::allowed_origins(&args.extension_ids)?;
    let context = ApplyContext {
        host: &host,
        origins: &origins,
        dry_run: args.dry_run,
    };

    println!("host binary: {}", host.display());
    println!("host log:    {}", log_path().display());
    let mut failed = 0;
    for target in &targets {
        let outcome = apply(target, action, &context);
        failed += usize::from(matches!(outcome, Outcome::Failed(_)));
        println!("{}", describe(target, &outcome));
    }
    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

/// Every place to register or unregister, for the current OS.
fn plan(args: &Args, browsers: &[Browser]) -> anyhow::Result<Vec<Target>> {
    if let Some(dir) = &args.user_data_dir {
        ensure!(
            !cfg!(windows),
            "--user-data-dir does not apply on Windows: browsers read the registry there"
        );
        ensure!(
            browsers.iter().any(|b| b.family() == Family::Chromium),
            "--user-data-dir only applies to Chromium-based browsers"
        );
        return Ok(vec![Target::file(
            "Chromium user-data-dir",
            Family::Chromium,
            &dir.join("NativeMessagingHosts"),
        )]);
    }
    if cfg!(target_os = "linux") {
        Ok(linux::targets(browsers, &home::real_home()?))
    } else if cfg!(target_os = "macos") {
        Ok(macos::targets(browsers, &home::real_home()?))
    } else if cfg!(windows) {
        Ok(windows::targets(browsers, &manifest_dir(args)?))
    } else {
        bail!("native messaging registration is not implemented for this operating system")
    }
}

fn manifest_dir(args: &Args) -> anyhow::Result<PathBuf> {
    if let Some(dir) = &args.manifest_dir {
        return Ok(dir.clone());
    }
    let local = std::env::var_os("LOCALAPPDATA").context("LOCALAPPDATA is not set")?;
    Ok(PathBuf::from(local)
        .join(crate::config::SLUG)
        .join("NativeMessagingHosts"))
}

/// The program browsers must start: the packaged app's execution alias when
/// running from MSIX (files under WindowsApps cannot be launched directly),
/// otherwise this very binary.
fn host_binary() -> anyhow::Result<PathBuf> {
    #[cfg(windows)]
    if let Some(alias) = crate::platform::windows::msix_alias_path() {
        return Ok(alias);
    }
    std::env::current_exe().context("cannot determine the path of this program")
}

/// One output line. Places that were not touched stay short: with every
/// channel of every browser, a full path on each would bury the ones that
/// matter.
fn describe(target: &Target, outcome: &Outcome) -> String {
    let label = &target.label;
    let path = target.location.describe();
    match outcome {
        Outcome::Written => format!("  ok       {label}  {path}"),
        Outcome::Removed => format!("  removed  {label}  {path}"),
        Outcome::DryRun => format!("  dry-run  {label}  {path}"),
        Outcome::NotPresent => format!("  absent   {label}"),
        Outcome::Skipped(reason) => format!("  skipped  {label}  ({reason})"),
        Outcome::Failed(reason) => format!("  FAILED   {label}  {path}  ({reason})"),
    }
}
