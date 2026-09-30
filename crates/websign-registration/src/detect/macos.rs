//! macOS: LaunchServices by bundle ID, which finds an app wherever it was
//! dragged (`/Applications`, `~/Applications`, elsewhere).

use super::{BrowserPackaging, InstalledBrowser, ordered};
use crate::browsers::Browser;

/// Bundle IDs per browser, stable channel first.
fn bundle_ids(browser: Browser) -> &'static [&'static str] {
    match browser {
        Browser::Chrome => &[
            "com.google.Chrome",
            "com.google.Chrome.beta",
            "com.google.Chrome.dev",
            "com.google.Chrome.canary",
        ],
        Browser::Chromium => &["org.chromium.Chromium"],
        Browser::Edge => &[
            "com.microsoft.edgemac",
            "com.microsoft.edgemac.Beta",
            "com.microsoft.edgemac.Dev",
            "com.microsoft.edgemac.Canary",
        ],
        Browser::Brave => &[
            "com.brave.Browser",
            "com.brave.Browser.beta",
            "com.brave.Browser.nightly",
        ],
        Browser::Vivaldi => &["com.vivaldi.Vivaldi", "com.vivaldi.Vivaldi.snapshot"],
        Browser::Opera => &[
            "com.operasoftware.Opera",
            "com.operasoftware.OperaNext",
            "com.operasoftware.OperaDeveloper",
            "com.operasoftware.OperaGX",
        ],
        Browser::Firefox => &[
            "org.mozilla.firefox",
            "org.mozilla.firefoxdeveloperedition",
            "org.mozilla.nightly",
        ],
    }
}

/// `find_bundle` answers whether an app with that bundle ID exists, and its
/// version. The first channel found names the version.
pub fn detect(find_bundle: &dyn Fn(&str) -> Option<Option<String>>) -> Vec<InstalledBrowser> {
    let found = Browser::ALL
        .into_iter()
        .filter_map(|browser| {
            let version = bundle_ids(browser).iter().find_map(|id| find_bundle(id))?;
            Some(InstalledBrowser {
                browser,
                version,
                packaging: BrowserPackaging::Native,
            })
        })
        .collect();
    ordered(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_later_channel_counts_and_carries_its_version() {
        let find = |id: &str| match id {
            "com.microsoft.edgemac.Dev" => Some(Some("131.0".to_owned())),
            "org.mozilla.firefox" => Some(None),
            _ => None,
        };
        assert_eq!(
            detect(&find),
            [
                InstalledBrowser {
                    browser: Browser::Edge,
                    version: Some("131.0".into()),
                    packaging: BrowserPackaging::Native,
                },
                InstalledBrowser {
                    browser: Browser::Firefox,
                    version: None,
                    packaging: BrowserPackaging::Native,
                },
            ]
        );
    }
}
