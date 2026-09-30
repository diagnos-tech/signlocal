//! Where macOS browsers look for user-level native messaging manifests:
//! `NativeMessagingHosts/` inside each browser's Application Support folder.
//! Opera reads Google Chrome's folder (it does on Linux, traced, and Opera's
//! developers said so for macOS), so an installed Opera also gets a manifest
//! there; its own folder is written too in case a release reads it.

use std::path::Path;

use super::browsers::{Browser, Family};
use super::destination::Target;

/// `(label, folder under ~/Library/Application Support)` per channel.
fn support_dirs(browser: Browser) -> &'static [(&'static str, &'static str)] {
    match browser {
        Browser::Chrome => &[
            ("Google Chrome", "Google/Chrome"),
            ("Google Chrome Beta", "Google/Chrome Beta"),
            ("Google Chrome Dev", "Google/Chrome Dev"),
            ("Google Chrome Canary", "Google/Chrome Canary"),
            ("Google Chrome for Testing", "Google/ChromeForTesting"),
            // Arc is Chromium-based and keeps its user data in `Arc/User
            // Data`, where Chromium looks for `NativeMessagingHosts`. It has
            // no `--browser` value of its own, so it rides with Chrome.
            ("Arc", "Arc/User Data"),
        ],
        Browser::Chromium => &[("Chromium", "Chromium")],
        Browser::Edge => &[
            ("Microsoft Edge", "Microsoft Edge"),
            ("Microsoft Edge Beta", "Microsoft Edge Beta"),
            ("Microsoft Edge Dev", "Microsoft Edge Dev"),
            ("Microsoft Edge Canary", "Microsoft Edge Canary"),
        ],
        Browser::Brave => &[
            ("Brave", "BraveSoftware/Brave-Browser"),
            ("Brave Beta", "BraveSoftware/Brave-Browser-Beta"),
            ("Brave Nightly", "BraveSoftware/Brave-Browser-Nightly"),
        ],
        Browser::Vivaldi => &[("Vivaldi", "Vivaldi")],
        Browser::Opera => &[
            ("Opera", "com.operasoftware.Opera"),
            ("Opera Beta", "com.operasoftware.OperaNext"),
            ("Opera Developer", "com.operasoftware.OperaDeveloper"),
            ("Opera GX", "com.operasoftware.OperaGX"),
        ],
        Browser::Firefox => &[("Firefox", "Mozilla")],
    }
}

/// `home` must be the real home directory. A sandboxed app has to ask the
/// user database for it, because `$HOME` points into its container.
pub fn targets(browsers: &[Browser], home: &Path) -> Vec<Target> {
    let support = home.join("Library/Application Support");
    let mut targets = Vec::new();
    for &browser in browsers {
        for &(label, dir) in support_dirs(browser) {
            let root = support.join(dir);
            targets.push(
                Target::file(label, browser.family(), &root.join("NativeMessagingHosts"))
                    .requiring(&root),
            );
            if browser == Browser::Opera {
                let chrome = support.join("Google/Chrome/NativeMessagingHosts");
                targets.push(
                    Target::file(
                        format!("{label} (Chrome's folder)"),
                        Family::Chromium,
                        &chrome,
                    )
                    .requiring(&root),
                );
            }
        }
    }
    targets
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::destination::Location;
    use crate::manifest;

    fn manifest_of(label: &str) -> (Family, PathBuf) {
        let target = targets(&Browser::ALL, Path::new("/Users/u"))
            .into_iter()
            .find(|t| t.label == label)
            .unwrap_or_else(|| panic!("no target labelled {label}"));
        let Location::File { manifest, .. } = target.location else {
            panic!("macOS registers files only");
        };
        (target.family, manifest)
    }

    #[test]
    fn chrome_edge_and_firefox_folders_match_the_vendors_documentation() {
        let name = manifest::file_name();
        let base = PathBuf::from("/Users/u/Library/Application Support");
        assert_eq!(
            manifest_of("Google Chrome").1,
            base.join("Google/Chrome/NativeMessagingHosts").join(&name)
        );
        assert_eq!(
            manifest_of("Microsoft Edge Canary").1,
            base.join("Microsoft Edge Canary/NativeMessagingHosts")
                .join(&name)
        );
        let (family, firefox) = manifest_of("Firefox");
        assert_eq!(family, Family::Firefox);
        assert_eq!(
            firefox,
            base.join("Mozilla/NativeMessagingHosts").join(&name)
        );
    }

    #[test]
    fn opera_also_writes_into_chromes_folder_once_its_own_exists() {
        let base = PathBuf::from("/Users/u/Library/Application Support");
        let gx = targets(&[Browser::Opera], Path::new("/Users/u"))
            .into_iter()
            .find(|t| t.label == "Opera GX (Chrome's folder)")
            .expect("Opera GX writes Chrome's folder");
        let Location::File {
            manifest, requires, ..
        } = gx.location
        else {
            panic!("files only");
        };
        assert_eq!(
            manifest,
            base.join("Google/Chrome/NativeMessagingHosts")
                .join(manifest::file_name())
        );
        assert_eq!(requires, Some(base.join("com.operasoftware.OperaGX")));
    }

    #[test]
    fn every_target_requires_the_browsers_own_folder() {
        for target in targets(&Browser::ALL, Path::new("/Users/u")) {
            let Location::File {
                manifest, requires, ..
            } = target.location
            else {
                panic!("files only");
            };
            if target.label.ends_with("(Chrome's folder)") {
                continue;
            }
            assert!(manifest.starts_with(requires.expect("must require its root")));
        }
    }
}
