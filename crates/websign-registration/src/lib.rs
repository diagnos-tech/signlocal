//! Making browsers find the app: native messaging host manifests, the
//! `websign:` URL scheme, extension pre-registration, and read-back status
//! for diagnostics.
//!
//! Registration writes only into folders a browser already created (a
//! `~/.config/vivaldi` for a machine without Vivaldi would litter the home
//! directory and mislead tools that treat the folder as an install). The app
//! registers again on every start, so a browser installed later is picked up.
//! On Windows, registry keys are written for every browser, since an unused
//! key is invisible.
//!
//! [`browsers`], [`destination`], the manifest renderer, the per-OS target
//! lists, the real-home lookup, [`run`] and [`plan`] are promoted from the
//! Phase-0 kit (`probe/src/nm/register`), where CI proved them on all three
//! systems, inside the macOS sandbox and from an MSIX package. [`detect`],
//! [`status`], [`url_scheme`], [`preregister`] and [`system`] are new
//! (`SPEC.md`).

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod browsers;
pub mod destination;
pub mod detect;
mod home;
mod linux;
mod macos;
mod manifest;
#[cfg(windows)]
pub mod msix;
pub mod preregister;
pub mod status;
pub mod system;
pub mod url_scheme;
mod windows;

use std::path::{Path, PathBuf};

pub use browsers::{Browser, BrowserChoice, Family};
pub use destination::{Action, Location, Outcome, Target};

/// Where manifests go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The current user's browser folders (`~/.config/...`,
    /// `~/Library/Application Support/...`, `HKCU`). What the app does on
    /// every start; needs no privileges.
    User,
    /// System-wide folders (`/etc/opt/chrome/native-messaging-hosts`,
    /// `/usr/lib/mozilla/native-messaging-hosts`, …). Only the Linux
    /// packages' post-install step uses it; it is the only place the Firefox
    /// Snap portal reads.
    System,
}

/// One registration run.
#[derive(Debug, Clone)]
pub struct Request {
    pub action: Action,
    pub scope: Scope,
    /// Empty = every browser.
    pub browsers: Vec<BrowserChoice>,
    /// Register only in `<DIR>/NativeMessagingHosts/` (Chromium started with
    /// `--user-data-dir=<DIR>`), for tests that must not touch real profiles.
    /// Not usable on Windows.
    pub user_data_dir: Option<PathBuf>,
    /// Extra Chromium extension IDs to allow (tests, forks).
    pub extension_ids: Vec<String>,
    /// Windows: folder for the manifest files the registry keys point to
    /// (default `%LOCALAPPDATA%\<slug>\NativeMessagingHosts`).
    pub manifest_dir: Option<PathBuf>,
    /// The program browsers must start; default [`host_binary`].
    pub host: Option<PathBuf>,
    /// Report what would be done, change nothing.
    pub dry_run: bool,
}

/// Why a run could not even be planned.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistrationError {
    #[error("{0}")]
    Unsupported(String),
    #[error("{0}")]
    Environment(String),
    #[error("invalid extension ID: {0}")]
    ExtensionId(String),
}

/// What happened at each target.
#[derive(Debug)]
pub struct Report {
    pub host: PathBuf,
    pub results: Vec<(Target, Outcome)>,
}

impl Report {
    /// Whether any target failed.
    pub fn failed(&self) -> bool {
        self.results
            .iter()
            .any(|(_, outcome)| matches!(outcome, Outcome::Failed(_)))
    }
}

/// Plans and applies `request`. Individual targets that fail are reported in
/// the [`Report`]; only planning problems are errors.
pub fn run(request: &Request) -> Result<Report, RegistrationError> {
    let targets = plan(request)?;
    let host = match &request.host {
        Some(host) => host.clone(),
        None => host_binary()?,
    };
    let origins = manifest::allowed_origins(&request.extension_ids)
        .map_err(|error| RegistrationError::ExtensionId(format!("{error:#}")))?;
    let context = destination::Context {
        host: &host,
        origins: &origins,
        dry_run: request.dry_run,
    };
    let results = targets
        .into_iter()
        .map(|target| {
            let outcome = destination::apply(&target, request.action, &context);
            (target, outcome)
        })
        .collect();
    Ok(Report { host, results })
}

/// Every place to register or unregister for `request` on this OS.
pub fn plan(request: &Request) -> Result<Vec<Target>, RegistrationError> {
    let browsers = browsers::selected(&request.browsers);
    if let Some(dir) = &request.user_data_dir {
        if cfg!(windows) {
            return Err(RegistrationError::Unsupported(
                "a user data dir does not apply on Windows: browsers read the registry there"
                    .into(),
            ));
        }
        if !browsers.iter().any(|b| b.family() == Family::Chromium) {
            return Err(RegistrationError::Unsupported(
                "a user data dir only applies to Chromium-based browsers".into(),
            ));
        }
        return Ok(vec![Target::file(
            "Chromium user-data-dir",
            Family::Chromium,
            &dir.join("NativeMessagingHosts"),
        )]);
    }
    if request.scope == Scope::System {
        if !cfg!(target_os = "linux") {
            return Err(RegistrationError::Unsupported(
                "system-wide registration is only used on Linux".into(),
            ));
        }
        let lib_dirs = [Path::new("/usr/lib"), Path::new("/usr/lib64")];
        return Ok(system::targets(&browsers, &lib_dirs));
    }
    if cfg!(target_os = "linux") {
        Ok(linux::targets(&browsers, &real_home()?))
    } else if cfg!(target_os = "macos") {
        Ok(macos::targets(&browsers, &real_home()?))
    } else if cfg!(windows) {
        Ok(windows::targets(&browsers, &manifest_dir(request)?))
    } else {
        Err(RegistrationError::Unsupported(
            "native messaging registration is not implemented for this operating system".into(),
        ))
    }
}

/// The program browsers must start: the MSIX execution alias when running
/// packaged (files under `WindowsApps` cannot be launched directly),
/// otherwise this very binary.
pub fn host_binary() -> Result<PathBuf, RegistrationError> {
    #[cfg(windows)]
    if let Some(alias) = msix::msix_alias_path() {
        return Ok(alias);
    }
    std::env::current_exe()
        .map_err(|e| RegistrationError::Environment(format!("cannot locate this program: {e}")))
}

fn real_home() -> Result<PathBuf, RegistrationError> {
    home::real_home().map_err(|error| RegistrationError::Environment(format!("{error:#}")))
}

fn manifest_dir(request: &Request) -> Result<PathBuf, RegistrationError> {
    if let Some(dir) = &request.manifest_dir {
        return Ok(dir.clone());
    }
    let local = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| RegistrationError::Environment("LOCALAPPDATA is not set".into()))?;
    Ok(PathBuf::from(local)
        .join(websign_project::SLUG)
        .join("NativeMessagingHosts"))
}
