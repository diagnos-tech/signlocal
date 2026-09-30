//! Manifest files: written only where the browser already has its folder,
//! removed together with any private host copy.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context as _;

use super::{Context, Outcome};
use crate::browsers::Family;
use crate::manifest;

pub fn install(
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

pub fn uninstall(
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
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("cannot remove {}", path.display()));
            }
        }
    }
    Ok(Outcome::Removed)
}
