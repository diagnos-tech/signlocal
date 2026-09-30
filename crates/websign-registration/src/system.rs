//! System-wide manifest locations on Linux, written by the deb/rpm
//! post-install step (`websign install --system`, as root).
//!
//! The Firefox Snap's WebExtensions portal reads only the host's manifests
//! (`~/.mozilla/...` or these), and distro Chrome and Chromium read them too.
//! Paths follow each browser's documented system locations
//! (`docs/research/native-messaging.md` §3.3). Flatpak browsers cannot see
//! `/etc` or `/usr/lib` of the host, so they get no system target; the app's
//! per-user registration on start covers them.
//!
//! Unlike per-user targets these are written unconditionally: a package is
//! installed before any browser may be, and an unused file under `/etc` is
//! harmless.

use std::path::{Path, PathBuf};

use crate::browsers::{Browser, Family};
use crate::destination::Target;

const CHROME: &str = "/etc/opt/chrome/native-messaging-hosts";

/// The `native-messaging-hosts` folders each Chromium browser reads under
/// `/etc`, in its reading order; the first is the one written. Brave, Opera
/// and Vivaldi read only Chrome's, Edge falls back to it (traced on Linux,
/// `docs/research/native-messaging.md` §3.6).
fn chromium_dirs(browser: Browser) -> &'static [&'static str] {
    match browser {
        Browser::Chrome | Browser::Brave | Browser::Opera | Browser::Vivaldi => &[CHROME],
        Browser::Chromium => &["/etc/chromium/native-messaging-hosts"],
        Browser::Edge => &["/etc/opt/edge/native-messaging-hosts", CHROME],
        Browser::Firefox => &[],
    }
}

/// System-wide targets for `browsers`, with `lib_dirs` the distro's library
/// roots (`/usr/lib`, `/usr/lib64`). Browsers sharing a folder share one
/// target, labelled with all their names.
pub fn targets(browsers: &[Browser], lib_dirs: &[&Path]) -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();
    for &browser in browsers {
        for dir in folders(browser, lib_dirs)
            .into_iter()
            .take(written(browser))
        {
            let candidate = Target::file(browser.label(), browser.family(), &dir);
            match targets
                .iter_mut()
                .find(|existing| existing.location == candidate.location)
            {
                Some(existing) => {
                    if !existing.label.contains(browser.label()) {
                        existing.label = format!("{}, {}", existing.label, browser.label());
                    }
                }
                None => targets.push(candidate),
            }
        }
    }
    targets
}

/// Every folder `browser` reads, in its reading order.
fn folders(browser: Browser, lib_dirs: &[&Path]) -> Vec<PathBuf> {
    match browser.family() {
        Family::Chromium => chromium_dirs(browser).iter().map(PathBuf::from).collect(),
        Family::Firefox => lib_dirs
            .iter()
            .map(|lib| lib.join("mozilla/native-messaging-hosts"))
            .collect(),
    }
}

/// How many of [`folders`] are written: Firefox reads every library root
/// (which one exists depends on the distro), a Chromium browser only needs
/// the first folder it reads.
fn written(browser: Browser) -> usize {
    match browser.family() {
        Family::Chromium => 1,
        Family::Firefox => usize::MAX,
    }
}

/// The system manifest paths `browser` reads, in its reading order.
pub(crate) fn manifests(browser: Browser, lib_dirs: &[&Path]) -> Vec<(Family, PathBuf)> {
    folders(browser, lib_dirs)
        .into_iter()
        .map(|dir| (browser.family(), dir.join(crate::manifest::file_name())))
        .collect()
}

#[cfg(test)]
mod tests;
