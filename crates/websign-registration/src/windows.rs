//! How Windows browsers find hosts: a `HKCU` key whose default value is the
//! path of a manifest file that can live anywhere.
//!
//! Chromium's own code reads `Software\Chromium` (only in Chromium-branded
//! builds) and then `Software\Google\Chrome` (`launch_context_win.cc`); Edge
//! documents its own key first. Brave, Vivaldi and Opera document nothing:
//! their Linux builds read Chrome's system folder (traced), and third parties
//! register Brave and Vivaldi under their own vendor keys, so those browsers
//! get every key they may read. Browsers try `HKCU` before `HKLM`, so a
//! per-user key is enough and needs no elevation.

use std::path::Path;

use super::browsers::{Browser, Family};
use super::destination::{Location, Target};
use websign_project::NATIVE_HOST;

const CHROME: &str = r"Software\Google\Chrome";
const CHROMIUM: &str = r"Software\Chromium";

/// The `HKCU\Software\...` folders whose `NativeMessagingHosts` `browser`
/// reads, first match wins.
fn vendor_keys(browser: Browser) -> &'static [&'static str] {
    match browser {
        Browser::Chrome => &[CHROME],
        Browser::Chromium => &[CHROMIUM, CHROME],
        Browser::Edge => &[r"Software\Microsoft\Edge", CHROMIUM, CHROME],
        Browser::Brave => &[r"Software\BraveSoftware\Brave-Browser", CHROMIUM, CHROME],
        Browser::Vivaldi => &[r"Software\Vivaldi", CHROMIUM, CHROME],
        Browser::Opera => &[CHROMIUM, CHROME],
        Browser::Firefox => &[r"Software\Mozilla"],
    }
}

/// How many of [`vendor_keys`] are written: a documented own key wins, so
/// Edge (and every single-key browser) needs one; for the others the
/// vendors publish nothing, so every key they may read is written.
fn written(browser: Browser) -> usize {
    match browser {
        Browser::Edge => 1,
        _ => usize::MAX,
    }
}

/// The keys `browser` reads, in its reading order.
pub(crate) fn lookup_order(browser: Browser) -> Vec<String> {
    vendor_keys(browser)
        .iter()
        .map(|vendor| host_key(vendor))
        .collect()
}

fn host_key(vendor: &str) -> String {
    format!(r"{vendor}\NativeMessagingHosts\{NATIVE_HOST}")
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
        for vendor in vendor_keys(browser).iter().take(written(browser)) {
            let subkey = host_key(vendor);
            match keys.iter_mut().find(|(existing, _, _)| *existing == subkey) {
                Some((_, labels, _)) => labels.push(browser.label()),
                None => keys.push((subkey, vec![browser.label()], browser.family())),
            }
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
        let chrome = format!(r"Software\Google\Chrome\NativeMessagingHosts\{NATIVE_HOST}");
        let shared: Vec<_> = keys.iter().filter(|(_, key)| *key == chrome).collect();
        assert_eq!(shared.len(), 1);
        assert_eq!(shared[0].0, "Google Chrome, Opera");
    }

    #[test]
    fn browsers_without_a_documented_key_get_every_key_they_may_read() {
        for browser in [Browser::Brave, Browser::Vivaldi, Browser::Opera] {
            let written: Vec<String> = keys(&[browser]).into_iter().map(|(_, key)| key).collect();
            assert_eq!(written, lookup_order(browser), "{browser:?}");
            assert!(
                written
                    .last()
                    .unwrap()
                    .starts_with(r"Software\Google\Chrome")
            );
        }
        let edge = keys(&[Browser::Edge]);
        assert_eq!(edge.len(), 1, "Edge's documented key wins");
        assert!(edge[0].1.starts_with(r"Software\Microsoft\Edge"));
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
