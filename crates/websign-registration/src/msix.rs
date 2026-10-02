//! Windows packaging facts. Promoted from the Phase-0 kit
//! (`probe/src/platform/windows.rs`), where the MSIX proof exercised it.

use std::path::PathBuf;

use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS};
use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFamilyName;
use windows::core::PWSTR;

use websign_project as config;

/// When running from an MSIX package, the app execution alias browsers must
/// start instead of the real binary (files under `WindowsApps` cannot be
/// launched directly). `None` when not packaged.
///
/// Windows puts the alias in `%LOCALAPPDATA%\Microsoft\WindowsApps` and in a
/// subfolder named after the package family. The subfolder copy is preferred:
/// it cannot be claimed by another package with the same alias.
pub fn msix_alias_path() -> Option<PathBuf> {
    let family = package_family_name()?;
    let aliases = PathBuf::from(std::env::var_os("LOCALAPPDATA")?)
        .join("Microsoft")
        .join("WindowsApps");
    let file = alias_file_name();
    let own = aliases.join(family).join(&file);
    // Aliases are reparse points that only open without being followed,
    // hence `symlink_metadata` instead of `exists`.
    if std::fs::symlink_metadata(&own).is_ok() {
        Some(own)
    } else {
        Some(aliases.join(file))
    }
}

/// Must match `uap5:ExecutionAlias` in `packaging/windows/msix`, which is
/// filled in from the same slug.
fn alias_file_name() -> String {
    format!("{}.exe", config::SLUG)
}

/// `None` when the process has no package identity.
fn package_family_name() -> Option<String> {
    let mut len = 0u32;
    // SAFETY: size query without a buffer; unpackaged processes get
    // APPMODEL_ERROR_NO_PACKAGE instead.
    if unsafe { GetCurrentPackageFamilyName(&mut len, None) } != ERROR_INSUFFICIENT_BUFFER {
        return None;
    }
    let mut name = vec![0u16; len as usize];
    // SAFETY: `name` holds `len` UTF-16 units, the size the first call asked for.
    if unsafe { GetCurrentPackageFamilyName(&mut len, Some(PWSTR(name.as_mut_ptr()))) }
        != ERROR_SUCCESS
    {
        return None;
    }
    // `len` counts the terminating NUL.
    name.truncate((len as usize).saturating_sub(1));
    String::from_utf16(&name).ok()
}
