//! Reads registrations back, for diagnostics ("Chrome can't find the app")
//! and for the repair button.
//!
//! Only manifests and registry values are read, never browser profiles.

use std::path::{Path, PathBuf};

use crate::browsers::{Browser, Family};

mod evaluate;
mod files;
mod registry;

/// The state of one browser's registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationState {
    /// A manifest exists and starts this app.
    Registered,
    /// A manifest exists but points elsewhere (an old install, another copy).
    PointsElsewhere { path: String },
    /// A manifest exists but is not valid JSON or lacks our extension IDs.
    Broken { reason: String },
    /// No manifest where this browser looks.
    Missing,
}

/// The registration state of `browser` for the current user (and, on Linux,
/// system-wide).
///
/// A browser with several channels or packagings (Chrome Beta, Flatpak
/// Chromium) counts as registered when any of them is; otherwise the first
/// problem found, in the order the browser reads its manifests, is reported.
pub fn registration_state(browser: Browser) -> RegistrationState {
    let host = match crate::host_binary() {
        Ok(host) => host,
        Err(error) => {
            return RegistrationState::Broken {
                reason: error.to_string(),
            };
        }
    };
    if cfg!(windows) {
        return registry::state(browser, &*crate::registry::system(), &host);
    }
    match crate::real_home() {
        Ok(home) => files::state(&candidates(browser, &home), &host),
        Err(error) => RegistrationState::Broken {
            reason: error.to_string(),
        },
    }
}

/// A manifest file a browser may read, and the program it must start.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    family: Family,
    manifest: PathBuf,
    /// A private copy of the host (Flatpak), or `None` for the app itself.
    host_copy: Option<PathBuf>,
}

/// Per-user locations first, then (Linux) the system ones.
fn candidates(browser: Browser, home: &Path) -> Vec<Candidate> {
    let user = if cfg!(target_os = "macos") {
        crate::macos::targets(&[browser], home)
    } else {
        crate::linux::targets(&[browser], home)
    };
    let mut all = file_candidates(user);
    if cfg!(target_os = "linux") {
        let libs = [Path::new("/usr/lib"), Path::new("/usr/lib64")];
        for (family, manifest) in crate::system::manifests(browser, &libs) {
            all.push(Candidate {
                family,
                manifest,
                host_copy: None,
            });
        }
    }
    all
}

/// The manifest files among `targets` (registry targets are skipped).
fn file_candidates(targets: Vec<crate::destination::Target>) -> Vec<Candidate> {
    targets
        .into_iter()
        .filter_map(|target| match target.location {
            crate::destination::Location::File {
                manifest,
                host_copy,
                ..
            } => Some(Candidate {
                family: target.family,
                manifest,
                host_copy,
            }),
            _ => None,
        })
        .collect()
}

/// Registered anywhere wins; else the first problem; else missing.
fn combine(states: impl IntoIterator<Item = RegistrationState>) -> RegistrationState {
    let mut first_problem = None;
    for state in states {
        match state {
            RegistrationState::Registered => return state,
            RegistrationState::Missing => {}
            problem => {
                first_problem.get_or_insert(problem);
            }
        }
    }
    first_problem.unwrap_or(RegistrationState::Missing)
}
