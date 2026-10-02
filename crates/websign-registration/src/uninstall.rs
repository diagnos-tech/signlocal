//! Which locations a partial uninstall may remove.
//!
//! Browsers share locations: on Windows Brave, Vivaldi and Opera write
//! Chromium's and Chrome's keys besides their own, on Linux and macOS Opera
//! reads Google Chrome's folder, and the Linux system folder of Chrome serves
//! Brave, Opera and Vivaldi too. Removing a location because one browser is
//! uninstalled would silently break every other browser that still relies on
//! it, so a location goes only when no remaining browser maps to it.

use std::collections::HashSet;

use crate::browsers::Browser;
use crate::destination::{Location, Target};

/// The targets `build` yields for `selected`, minus every location that a
/// browser outside `selected` also uses. Selecting every browser removes
/// everything.
pub(crate) fn exclusive(
    selected: &[Browser],
    build: impl Fn(&[Browser]) -> Vec<Target>,
) -> Vec<Target> {
    let remaining: Vec<Browser> = Browser::ALL
        .into_iter()
        .filter(|browser| !selected.contains(browser))
        .collect();
    let removed = build(selected);
    if remaining.is_empty() {
        return removed;
    }
    without_shared(removed, &build(&remaining))
}

/// `removed` without the targets whose location is also in `remaining`.
fn without_shared(removed: Vec<Target>, remaining: &[Target]) -> Vec<Target> {
    let kept: HashSet<String> = remaining.iter().map(|t| identity(&t.location)).collect();
    removed
        .into_iter()
        .filter(|target| !kept.contains(&identity(&target.location)))
        .collect()
}

/// What makes two locations the same place: the manifest path compared by
/// components (so `a/b` and `a\b` agree on Windows), or the registry key
/// (whose names are case-insensitive).
fn identity(location: &Location) -> String {
    match location {
        Location::File { manifest, .. } => {
            let parts: Vec<_> = manifest
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect();
            format!("file:{}", parts.join("/"))
        }
        Location::Registry { subkey, .. } => format!("key:{}", subkey.to_lowercase()),
    }
}

#[cfg(test)]
mod tests;
