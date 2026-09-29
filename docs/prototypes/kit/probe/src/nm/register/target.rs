//! One place a browser looks for the host, and how to write or remove it.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context as _;

use super::browsers::Family;
use super::manifest;

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

pub fn apply(target: &Target, action: Action, context: &Context<'_>) -> Outcome {
    let result = match (&target.location, action) {
        (
            Location::File {
                manifest,
                requires,
                host_copy,
            },
            Action::Install,
        ) => install_file(
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
        ) => uninstall_file(manifest, host_copy.as_deref(), context.dry_run),
        (Location::Registry { subkey, manifest }, Action::Install) => {
            registry::set(subkey, manifest, context.dry_run)
        }
        (Location::Registry { subkey, .. }, Action::Uninstall) => {
            registry::remove(subkey, context.dry_run)
        }
    };
    result.unwrap_or_else(|error| Outcome::Failed(format!("{error:#}")))
}

fn install_file(
    family: Family,
    manifest_path: &Path,
    requires: Option<&Path>,
    host_copy: Option<&Path>,
    context: &Context<'_>,
) -> anyhow::Result<Outcome> {
    if let Some(root) = requires.filter(|root| !root.exists()) {
        return Ok(Outcome::Skipped(format!("{} not found", root.display())));
    }
    if context.dry_run {
        return Ok(Outcome::DryRun);
    }
    let parent = manifest_path
        .parent()
        .context("manifest path has no folder")?;
    fs::create_dir_all(parent).with_context(|| format!("cannot create {}", parent.display()))?;
    let host = match host_copy {
        Some(copy) => {
            fs::copy(context.host, copy)
                .with_context(|| format!("cannot copy the host to {}", copy.display()))?;
            copy
        }
        None => context.host,
    };
    let text = manifest::render(family, host, context.origins)?;
    fs::write(manifest_path, text)
        .with_context(|| format!("cannot write {}", manifest_path.display()))?;
    Ok(Outcome::Written)
}

fn uninstall_file(
    manifest_path: &Path,
    host_copy: Option<&Path>,
    dry_run: bool,
) -> anyhow::Result<Outcome> {
    let present = manifest_path.exists() || host_copy.is_some_and(Path::exists);
    if !present {
        return Ok(Outcome::NotPresent);
    }
    if dry_run {
        return Ok(Outcome::DryRun);
    }
    for path in std::iter::once(manifest_path).chain(host_copy) {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("cannot remove {}", path.display()));
            }
        }
    }
    Ok(Outcome::Removed)
}

mod registry;

#[cfg(test)]
mod tests;
