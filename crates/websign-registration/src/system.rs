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
use crate::destination::{Location, Target};

/// The `native-messaging-hosts` folder each Chromium browser reads under
/// `/etc`. Brave and Opera have no folder of their own and read Chrome's.
fn chromium_dir(browser: Browser) -> Option<&'static str> {
    Some(match browser {
        Browser::Chrome | Browser::Brave | Browser::Opera => {
            "/etc/opt/chrome/native-messaging-hosts"
        }
        Browser::Chromium => "/etc/chromium/native-messaging-hosts",
        Browser::Edge => "/etc/opt/edge/native-messaging-hosts",
        Browser::Vivaldi => "/etc/opt/vivaldi/native-messaging-hosts",
        Browser::Firefox => return None,
    })
}

/// System-wide targets for `browsers`, with `lib_dirs` the distro's library
/// roots (`/usr/lib`, `/usr/lib64`). Browsers sharing a folder share one
/// target, labelled with all their names.
pub fn targets(browsers: &[Browser], lib_dirs: &[&Path]) -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();
    for &browser in browsers {
        let dirs: Vec<PathBuf> = match chromium_dir(browser) {
            Some(dir) => vec![PathBuf::from(dir)],
            None => lib_dirs
                .iter()
                .map(|lib| lib.join("mozilla/native-messaging-hosts"))
                .collect(),
        };
        for dir in dirs {
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

/// The system manifest paths `browser` reads, in its reading order.
pub(crate) fn manifests(browser: Browser, lib_dirs: &[&Path]) -> Vec<(Family, PathBuf)> {
    targets(&[browser], lib_dirs)
        .into_iter()
        .filter_map(|target| match target.location {
            Location::File { manifest, .. } => Some((target.family, manifest)),
            Location::Registry { .. } => None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
