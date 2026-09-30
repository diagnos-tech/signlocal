//! Which browsers are installed, for the diagnostics Browsers tab.
//!
//! Detected from install locations, the registry (Windows `StartMenuInternet`
//! and `App Paths`) and LaunchServices (macOS), never from profile folders
//! (profile folders hold personal data; the app never reads them).

use crate::browsers::Browser;

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

/// Every supported browser installed for this user or system-wide.
pub fn installed_browsers() -> Vec<InstalledBrowser> {
    todo!("SPEC.md §2")
}
