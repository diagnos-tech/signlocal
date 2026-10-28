//! One place a browser looks for the host, and how to write or remove it.

use std::path::{Path, PathBuf};

use super::browsers::Family;
use super::manifest;
use crate::registry::Registry;

/// A registration destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// For people: `"Google Chrome"`, `"Firefox (Flatpak)"`.
    pub label: String,
    pub family: Family,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// A manifest file.
    File {
        manifest: PathBuf,
        /// A directory that must already exist for the browser to count as
        /// installed. `None` writes unconditionally.
        ///
        /// Creating another vendor's configuration folder for a browser that
        /// is not there litters the home directory, and some tools read a
        /// present folder as "browser installed".
        requires: Option<PathBuf>,
        /// Flatpak browsers cannot see the host's binary, so a copy is placed
        /// here and the manifest points to it.
        host_copy: Option<PathBuf>,
    },
    /// A `HKCU` key whose default value is the path of a manifest file
    /// (Windows browsers find hosts through the registry).
    Registry { subkey: String, manifest: PathBuf },
}

impl Target {
    /// A manifest file inside `hosts_dir`, named as browsers expect.
    pub fn file(label: impl Into<String>, family: Family, hosts_dir: &Path) -> Self {
        Self {
            label: label.into(),
            family,
            location: Location::File {
                manifest: hosts_dir.join(manifest::file_name()),
                requires: None,
                host_copy: None,
            },
        }
    }

    /// Only writes when `root` exists: see [`Location::File::requires`].
    pub fn requiring(mut self, root: &Path) -> Self {
        if let Location::File { requires, .. } = &mut self.location {
            *requires = Some(root.to_owned());
        }
        self
    }

    /// Points the manifest at a private copy of the host binary.
    pub fn with_host_copy(mut self, copy: PathBuf) -> Self {
        if let Location::File { host_copy, .. } = &mut self.location {
            *host_copy = Some(copy);
        }
        self
    }
}

impl Location {
    /// Where the registration lives, for output.
    pub fn describe(&self) -> String {
        match self {
            Self::File { manifest, .. } => manifest.display().to_string(),
            Self::Registry { subkey, .. } => format!("HKCU\\{subkey}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Install,
    Uninstall,
}

/// What was done, or would be.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Written,
    Removed,
    /// Uninstall found nothing to remove.
    NotPresent,
    Skipped(String),
    /// `--dry-run`: the action was not performed.
    DryRun,
    Failed(String),
}

/// Everything an action needs besides the target.
#[derive(Debug)]
pub struct Context<'a> {
    /// The binary browsers should start.
    pub host: &'a Path,
    pub origins: &'a [String],
    pub dry_run: bool,
}

/// Applies `action` at `target` against this machine's registry.
pub fn apply(target: &Target, action: Action, context: &Context<'_>) -> Outcome {
    apply_with(target, action, context, &*crate::registry::system())
}

/// [`apply`] against `registry`, so Windows targets can be exercised on any
/// OS.
pub fn apply_with(
    target: &Target,
    action: Action,
    context: &Context<'_>,
    registry: &dyn Registry,
) -> Outcome {
    let result = match (&target.location, action) {
        (
            Location::File {
                manifest,
                requires,
                host_copy,
            },
            Action::Install,
        ) => file::install(
            target.family,
            manifest,
            requires.as_deref(),
            host_copy.as_deref(),
            context,
        ),
        (
            Location::File {
                manifest,
                host_copy,
                ..
            },
            Action::Uninstall,
        ) => file::uninstall(manifest, host_copy.as_deref(), context.dry_run),
        (Location::Registry { subkey, manifest }, Action::Install) => {
            registry::set(registry, subkey, manifest, context.dry_run)
        }
        (Location::Registry { subkey, .. }, Action::Uninstall) => {
            registry::remove(registry, subkey, context.dry_run)
        }
    };
    result.unwrap_or_else(|error| Outcome::Failed(format!("{error:#}")))
}

mod file;
mod registry;

#[cfg(test)]
mod registry_tests;
#[cfg(test)]
mod tests;
