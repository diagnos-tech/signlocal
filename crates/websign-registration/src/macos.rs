//! Where macOS browsers look for user-level native messaging manifests:
//! `NativeMessagingHosts/` inside each browser's Application Support folder.

use std::path::Path;

use super::browsers::Browser;
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
            ("Opera Developer", "com.operasoftware.OperaDeveloper"),
        ],
        Browser::Firefox => &[("Firefox", "Mozilla")],
    }
}

/// `home` must be the real home directory. A sandboxed app has to ask the
/// user database for it, because `$HOME` points into its container.
pub fn targets(browsers: &[Browser], home: &Path) -> Vec<Target> {
    let support = home.join("Library/Application Support");
    browsers
        .iter()
        .flat_map(|&browser| {
            support_dirs(browser)
                .iter()
                .map(move |&(label, dir)| (browser, label, dir))
        })
        .map(|(browser, label, dir)| {
            let root = support.join(dir);
            Target::file(label, browser.family(), &root.join("NativeMessagingHosts"))
                .requiring(&root)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::browsers::Family;
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
    fn every_target_requires_the_browsers_own_folder() {
        for target in targets(&Browser::ALL, Path::new("/Users/u")) {
            let Location::File {
                manifest, requires, ..
            } = target.location
            else {
                panic!("files only");
            };
            assert!(manifest.starts_with(requires.expect("must require its root")));
        }
    }
}
