//! Windows registry entries that point browsers at a host manifest.

use std::path::Path;

use anyhow::Context as _;

use super::Outcome;
use crate::registry::{Hive, Registry};

/// Sets the default value of `HKCU\<subkey>` to the manifest's path.
pub fn set(
    registry: &dyn Registry,
    subkey: &str,
    manifest: &Path,
    dry_run: bool,
) -> anyhow::Result<Outcome> {
    if dry_run {
        return Ok(Outcome::DryRun);
    }
    registry
        .set_string(subkey, "", &manifest.to_string_lossy())
        .with_context(|| format!("cannot set HKCU\\{subkey}"))?;
    Ok(Outcome::Written)
}

/// Removes `HKCU\<subkey>` and everything under it.
pub fn remove(registry: &dyn Registry, subkey: &str, dry_run: bool) -> anyhow::Result<Outcome> {
    if !registry.key_exists(Hive::CurrentUser, subkey) {
        return Ok(Outcome::NotPresent);
    }
    if dry_run {
        return Ok(Outcome::DryRun);
    }
    registry
        .remove_tree(subkey)
        .with_context(|| format!("cannot remove HKCU\\{subkey}"))?;
    Ok(Outcome::Removed)
}
