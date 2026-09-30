//! How Windows browsers find hosts: a `HKCU` key per browser whose default
//! value is the path of a manifest file that can live anywhere.
//!
//! Chrome, Opera and every Chrome-derived browser without a key of its own
//! read Chrome's; Edge, Chromium, Brave, Vivaldi and Firefox have theirs.
//! Chrome tries `HKCU` before `HKLM`, so a per-user key is enough and needs
//! no elevation.

use std::path::Path;

use super::browsers::{Browser, Family};
use super::destination::{Location, Target};
use websign_project::NATIVE_HOST;

/// The `HKCU\Software\...` folder holding a browser's `NativeMessagingHosts`.
fn vendor_key(browser: Browser) -> &'static str {
    match browser {
        Browser::Chrome | Browser::Opera => r"Software\Google\Chrome",
        Browser::Chromium => r"Software\Chromium",
        Browser::Edge => r"Software\Microsoft\Edge",
        Browser::Brave => r"Software\BraveSoftware\Brave-Browser",
        Browser::Vivaldi => r"Software\Vivaldi",
        Browser::Firefox => r"Software\Mozilla",
    }
}

/// The keys `browser` reads, first match wins: its own, then the ones it
/// documents falling back to (only Edge: Chromium's, then Chrome's). An
/// undocumented fallback is left out on purpose: counting a key the browser
/// may not read would report `Registered` with nothing to repair.
pub(crate) fn lookup_order(browser: Browser) -> Vec<String> {
    let vendors: &[Browser] = match browser {
        Browser::Edge => &[Browser::Chromium, Browser::Chrome],
        _ => &[],
    };
    std::iter::once(browser)
        .chain(vendors.iter().copied())
        .map(host_key)
        .collect()
}

fn host_key(browser: Browser) -> String {
    format!(
        r"{}\NativeMessagingHosts\{NATIVE_HOST}",
        vendor_key(browser)
    )
}

/// Manifest files first (the keys point at them), then one key per distinct
/// vendor folder.
pub fn targets(browsers: &[Browser], manifest_dir: &Path) -> Vec<Target> {
    let mut targets = Vec::new();
    let manifest_for = |family: Family| {
        let suffix = match family {
            Family::Chromium => "chromium",
            Family::Firefox => "firefox",
        };
        manifest_dir.join(format!("{NATIVE_HOST}.{suffix}.json"))
    };
    for family in [Family::Chromium, Family::Firefox] {
        if browsers.iter().any(|browser| browser.family() == family) {
            let manifest = manifest_for(family);
            targets.push(Target {
                label: format!("manifest ({family:?} family)"),
                family,
                location: Location::File {
                    manifest,
                    requires: None,
                    host_copy: None,
                },
            });
        }
    }

    let mut keys: Vec<(String, Vec<&str>, Family)> = Vec::new();
    for &browser in browsers {
        let subkey = host_key(browser);
        match keys.iter_mut().find(|(existing, _, _)| *existing == subkey) {
            Some((_, labels, _)) => labels.push(browser.label()),
            None => keys.push((subkey, vec![browser.label()], browser.family())),
        }
    }
    for (subkey, labels, family) in keys {
        targets.push(Target {
            label: labels.join(", "),
            family,
            location: Location::Registry {
                subkey,
                manifest: manifest_for(family),
            },
        });
    }
    targets
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn keys(browsers: &[Browser]) -> Vec<(String, String)> {
        targets(browsers, Path::new(r"C:\Users\u\AppData\Local\websign"))
            .into_iter()
            .filter_map(|target| match target.location {
                Location::Registry { subkey, .. } => Some((target.label, subkey)),
                Location::File { .. } => None,
            })
            .collect()
    }

    #[test]
    fn chrome_and_opera_share_chromes_key() {
        let keys = keys(&[Browser::Chrome, Browser::Opera]);
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].0, "Google Chrome, Opera");
        assert_eq!(
            keys[0].1,
            format!(r"Software\Google\Chrome\NativeMessagingHosts\{NATIVE_HOST}")
        );
    }

    #[test]
    fn each_vendor_gets_its_documented_key() {
        let all = keys(&Browser::ALL);
        let subkeys: Vec<&str> = all.iter().map(|(_, key)| key.as_str()).collect();
        for vendor in [
            r"Software\Chromium",
            r"Software\Microsoft\Edge",
            r"Software\BraveSoftware\Brave-Browser",
            r"Software\Vivaldi",
            r"Software\Mozilla",
        ] {
            let expected = format!(r"{vendor}\NativeMessagingHosts\{NATIVE_HOST}");
            assert!(subkeys.contains(&expected.as_str()), "missing {expected}");
        }
    }

    #[test]
    fn firefox_keys_point_at_the_firefox_manifest() {
        let all = targets(&[Browser::Firefox, Browser::Edge], Path::new(r"C:\m"));
        for target in all {
            if let Location::Registry { subkey, manifest } = target.location {
                let firefox = subkey.starts_with(r"Software\Mozilla");
                let suffix = if firefox { "firefox" } else { "chromium" };
                assert_eq!(
                    manifest,
                    PathBuf::from(r"C:\m").join(format!("{NATIVE_HOST}.{suffix}.json"))
                );
            }
        }
    }

    #[test]
    fn manifest_files_precede_the_keys_that_point_at_them() {
        let all = targets(&Browser::ALL, Path::new(r"C:\m"));
        let first_key = all
            .iter()
            .position(|t| matches!(t.location, Location::Registry { .. }))
            .unwrap();
        assert!(
            all[..first_key]
                .iter()
                .all(|t| matches!(t.location, Location::File { .. }))
        );
        assert_eq!(first_key, 2, "one manifest per family");
    }
}
