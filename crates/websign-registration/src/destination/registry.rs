//! Windows registry entries that point browsers at a host manifest.

use std::path::Path;

use super::Outcome;

/// Sets the default value of `HKCU\<subkey>` to the manifest's path.
#[cfg(windows)]
pub fn set(subkey: &str, manifest: &Path, dry_run: bool) -> anyhow::Result<Outcome> {
    use anyhow::Context as _;

    if dry_run {
        return Ok(Outcome::DryRun);
    }
    let key = windows_registry::CURRENT_USER
        .create(subkey)
        .with_context(|| format!("cannot create HKCU\\{subkey}"))?;
    key.set_string("", manifest.to_string_lossy())
        .with_context(|| format!("cannot set the value of HKCU\\{subkey}"))?;
    Ok(Outcome::Written)
}

/// Removes `HKCU\<subkey>` and everything under it.
#[cfg(windows)]
pub fn remove(subkey: &str, dry_run: bool) -> anyhow::Result<Outcome> {
    use anyhow::Context as _;

    if windows_registry::CURRENT_USER.open(subkey).is_err() {
        return Ok(Outcome::NotPresent);
    }
    if dry_run {
        return Ok(Outcome::DryRun);
    }
    windows_registry::CURRENT_USER
        .remove_tree(subkey)
        .with_context(|| format!("cannot remove HKCU\\{subkey}"))?;
    Ok(Outcome::Removed)
}

#[cfg(not(windows))]
pub fn set(_: &str, _: &Path, _: bool) -> anyhow::Result<Outcome> {
    anyhow::bail!("the registry only exists on Windows")
}

#[cfg(not(windows))]
pub fn remove(_: &str, _: bool) -> anyhow::Result<Outcome> {
    anyhow::bail!("the registry only exists on Windows")
}
