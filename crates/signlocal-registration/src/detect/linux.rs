//! Linux: executables on `PATH` and under `/opt`, Snap launchers in
//! `/snap/bin`, and Flatpak application IDs.

use std::fs;
use std::path::{Path, PathBuf};

use super::{BrowserPackaging, InstalledBrowser, ordered};
use crate::browsers::Browser;

/// Launcher names per browser, every channel included.
fn executables(browser: Browser) -> &'static [&'static str] {
    match browser {
        Browser::Chrome => &[
            "google-chrome",
            "google-chrome-stable",
            "google-chrome-beta",
            "google-chrome-unstable",
            "google-chrome-canary",
        ],
        Browser::Chromium => &["chromium", "chromium-browser"],
        Browser::Edge => &[
            "microsoft-edge",
            "microsoft-edge-stable",
            "microsoft-edge-beta",
            "microsoft-edge-dev",
        ],
        Browser::Brave => &[
            "brave-browser",
            "brave-browser-beta",
            "brave-browser-nightly",
        ],
        Browser::Vivaldi => &["vivaldi", "vivaldi-stable", "vivaldi-snapshot"],
        Browser::Opera => &["opera", "opera-beta", "opera-developer"],
        Browser::Firefox => &["firefox", "firefox-esr", "firefox-developer-edition"],
    }
}

/// Vendor packages that install under `/opt` without a `PATH` link.
fn opt_binaries(browser: Browser) -> &'static [&'static str] {
    match browser {
        Browser::Chrome => &["opt/google/chrome/chrome", "opt/google/chrome-beta/chrome"],
        Browser::Edge => &["opt/microsoft/msedge/msedge"],
        Browser::Brave => &["opt/brave.com/brave/brave"],
        Browser::Vivaldi => &["opt/vivaldi/vivaldi"],
        Browser::Firefox => &["opt/firefox/firefox"],
        Browser::Chromium | Browser::Opera => &[],
    }
}

/// Snap names; Chrome and Edge publish no snap.
fn snap(browser: Browser) -> Option<&'static str> {
    match browser {
        Browser::Chromium => Some("chromium"),
        Browser::Firefox => Some("firefox"),
        Browser::Brave => Some("brave"),
        Browser::Opera => Some("opera"),
        Browser::Vivaldi => Some("vivaldi"),
        Browser::Chrome | Browser::Edge => None,
    }
}

/// The directories of `$PATH`.
pub fn path_dirs() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default()
}

/// `root` is `/` except in tests; `path_dirs` and `flatpak_apps` come from
/// [`path_dirs`] and `flatpak list`.
pub fn detect(
    root: &Path,
    path_dirs: &[PathBuf],
    flatpak_apps: &[String],
) -> Vec<InstalledBrowser> {
    let snap_bin = root.join("snap/bin");
    let native_dirs: Vec<&PathBuf> = path_dirs.iter().filter(|dir| **dir != snap_bin).collect();
    let mut found = Vec::new();
    let mut add = |browser, packaging| {
        found.push(InstalledBrowser {
            browser,
            version: None,
            packaging,
        });
    };
    for browser in Browser::ALL {
        let on_path = executables(browser)
            .iter()
            .flat_map(|name| native_dirs.iter().map(move |dir| dir.join(name)))
            .any(|path| is_native_executable(&path));
        let in_opt = opt_binaries(browser)
            .iter()
            .any(|relative| is_native_executable(&root.join(relative)));
        if on_path || in_opt {
            add(browser, BrowserPackaging::Native);
        }
        if snap(browser).is_some_and(|name| is_executable(&snap_bin.join(name))) {
            add(browser, BrowserPackaging::Snap);
        }
        let flatpak = crate::linux::flatpak(browser).map(|(app_id, _)| app_id);
        if flatpak.is_some_and(|id| flatpak_apps.iter().any(|app| app == id)) {
            add(browser, BrowserPackaging::Flatpak);
        }
    }
    ordered(found)
}

/// An executable that is not Ubuntu's transitional script forwarding to the
/// snap (`/usr/bin/firefox`, `/usr/bin/chromium-browser`), which would
/// otherwise report a snap browser twice.
fn is_native_executable(path: &Path) -> bool {
    const SCRIPT_LIMIT: u64 = 16 * 1024;
    if !is_executable(path) {
        return false;
    }
    let small = fs::metadata(path).is_ok_and(|meta| meta.len() <= SCRIPT_LIMIT);
    if !small {
        return true;
    }
    let text = fs::read(path).unwrap_or_default();
    let text = String::from_utf8_lossy(&text);
    !(text.starts_with("#!") && text.contains("/snap/bin/"))
}

fn is_executable(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        meta.is_file() && meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    meta.is_file()
}

#[cfg(test)]
mod tests;
