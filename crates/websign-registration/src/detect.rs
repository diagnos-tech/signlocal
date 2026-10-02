//! Which browsers are installed, for the diagnostics Browsers tab and for
//! registration (a browser never started has no folder yet).
//!
//! Detected from install locations, the registry (Windows `StartMenuInternet`
//! and `App Paths`) and LaunchServices (macOS), never from profile folders
//! (profile folders hold personal data; the app never reads them).

use std::path::Path;

use crate::browsers::Browser;

#[cfg(windows)]
mod file_version;
#[cfg(target_os = "macos")]
mod launch_services;
mod linux;
mod macos;
mod platform;
mod windows;

/// A browser found on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledBrowser {
    pub browser: Browser,
    /// From the executable's version resource / `Info.plist` / `--version`
    /// when cheap to read.
    pub version: Option<String>,
    pub packaging: BrowserPackaging,
}

/// How the browser was installed; decides where it reads manifests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserPackaging {
    Native,
    /// Linux Snap: Firefox asks the WebExtensions portal, Chromium reads
    /// `~/snap/chromium/common/...`.
    Snap,
    /// Linux Flatpak: needs a host copy inside the sandbox.
    Flatpak,
}

/// Every supported browser installed for this user or system-wide, in
/// [`Browser::ALL`] order.
pub fn installed_browsers() -> Vec<InstalledBrowser> {
    if cfg!(windows) {
        windows::detect(&*crate::registry::system(), &platform::file_version)
    } else if cfg!(target_os = "macos") {
        macos::detect(&platform::find_bundle)
    } else {
        linux::detect(
            Path::new("/"),
            &linux::path_dirs(),
            &platform::flatpak_apps(),
        )
    }
}

/// Browsers installed as ordinary programs (not Snap or Flatpak, which
/// registration reaches through their own folders), for
/// [`crate::presence`]. Linux skips `flatpak list`: only native installs
/// matter here.
pub(crate) fn native_browsers() -> Vec<Browser> {
    let found = if cfg!(target_os = "macos") {
        macos::detect(&platform::find_bundle)
    } else {
        linux::detect(Path::new("/"), &linux::path_dirs(), &[])
    };
    found
        .into_iter()
        .filter(|entry| entry.packaging == BrowserPackaging::Native)
        .map(|entry| entry.browser)
        .collect()
}

/// One entry per browser and packaging, in [`Browser::ALL`] order; the first
/// sighting of a pair keeps its version.
fn ordered(found: Vec<InstalledBrowser>) -> Vec<InstalledBrowser> {
    let mut result: Vec<InstalledBrowser> = Vec::new();
    for browser in Browser::ALL {
        for entry in found.iter().filter(|entry| entry.browser == browser) {
            let seen = result
                .iter()
                .any(|kept| kept.browser == browser && kept.packaging == entry.packaging);
            if !seen {
                result.push(entry.clone());
            }
        }
    }
    result
}
