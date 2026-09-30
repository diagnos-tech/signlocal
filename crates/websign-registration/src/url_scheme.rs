//! The `websign:` URL scheme, so the website's `/activate` page can start the
//! app once to register it with browsers (stores run no install scripts).
//!
//! Direct builds register it per user: `HKCU\Software\Classes\websign`
//! (Windows), a `.desktop` file with `x-scheme-handler/websign` (Linux). On
//! macOS the `.app`'s `CFBundleURLTypes` declares it; nothing to write.
//! Packaged builds declare it in their manifests.

use std::path::{Path, PathBuf};

use crate::destination::Outcome;

mod linux;
mod macos;
mod windows;

pub use macos::info_plist_url_types;

/// Registers `executable` as the handler of `websign:` URLs.
pub fn register(executable: &Path, dry_run: bool) -> Outcome {
    if cfg!(windows) {
        windows::register(&*crate::registry::system(), executable, dry_run)
    } else if cfg!(target_os = "macos") {
        macos::skipped()
    } else {
        match applications_dir() {
            Ok(dir) => linux::register(&dir, executable, dry_run, &linux::xdg_mime),
            Err(outcome) => outcome,
        }
    }
}

/// Removes the per-user handler registration.
pub fn unregister(dry_run: bool) -> Outcome {
    if cfg!(windows) {
        windows::unregister(&*crate::registry::system(), dry_run)
    } else if cfg!(target_os = "macos") {
        macos::skipped()
    } else {
        match applications_dir() {
            Ok(dir) => linux::unregister(&dir, dry_run),
            Err(outcome) => outcome,
        }
    }
}

/// `$XDG_DATA_HOME/applications`, by default `~/.local/share/applications`.
fn applications_dir() -> Result<PathBuf, Outcome> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute());
    let data_home = match data_home {
        Some(dir) => dir,
        None => crate::real_home()
            .map_err(|error| Outcome::Failed(error.to_string()))?
            .join(".local/share"),
    };
    Ok(data_home.join("applications"))
}
