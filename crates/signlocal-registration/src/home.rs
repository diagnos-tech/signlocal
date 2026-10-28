//! The home directory browsers keep their folders in.

use std::path::PathBuf;

use anyhow::Context as _;

/// The user's real home directory.
///
/// Inside the macOS App Sandbox `$HOME` is the app's container
/// (`~/Library/Containers/<bundle id>/Data`), where no browser looks, and the
/// sandbox's temporary-exception entitlements are relative to the real home.
/// So on macOS the home comes from the user database, which the sandbox does
/// not redirect; `$HOME` is only the fallback.
pub fn real_home() -> anyhow::Result<PathBuf> {
    #[cfg(target_os = "macos")]
    if let Some(home) = user_database_home() {
        return Ok(home);
    }
    std::env::home_dir().context("cannot determine the home directory")
}

/// `pw_dir` of the current user, from `getpwuid_r`.
#[cfg(target_os = "macos")]
fn user_database_home() -> Option<PathBuf> {
    use std::ffi::{CStr, OsStr};
    use std::mem::MaybeUninit;
    use std::os::unix::ffi::OsStrExt;

    const MAX_BUFFER: usize = 1 << 20;
    let mut buffer = vec![0u8; 4096];
    loop {
        let mut entry = MaybeUninit::<libc::passwd>::uninit();
        let mut result: *mut libc::passwd = std::ptr::null_mut();
        // SAFETY: `entry` and `result` are live out-pointers, and the strings
        // of the entry are copied into `buffer`, whose real length is passed.
        let status = unsafe {
            libc::getpwuid_r(
                libc::getuid(),
                entry.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if status == libc::ERANGE && buffer.len() < MAX_BUFFER {
            buffer.resize(buffer.len() * 2, 0);
            continue;
        }
        if status != 0 || result.is_null() {
            return None;
        }
        // SAFETY: on success `result` points to `entry`, now initialized.
        let directory = unsafe { (*result).pw_dir };
        if directory.is_null() {
            return None;
        }
        // SAFETY: `pw_dir` is a NUL-terminated string inside `buffer`, which
        // is alive and unchanged here; the bytes are copied out below.
        let bytes = unsafe { CStr::from_ptr(directory) }.to_bytes();
        return (!bytes.is_empty()).then(|| PathBuf::from(OsStr::from_bytes(bytes)));
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn the_user_database_names_an_absolute_home_outside_any_container() {
        let home = user_database_home().expect("getpwuid_r must know the current user");
        assert!(home.is_absolute(), "{}", home.display());
        assert!(!home.to_string_lossy().contains("/Library/Containers/"));
    }
}
