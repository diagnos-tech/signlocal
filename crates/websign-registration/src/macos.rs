//! Where macOS browsers look for user-level native messaging manifests:
//! `NativeMessagingHosts/` inside each browser's Application Support folder,
//! with two exceptions. Brave reads Google Chrome's folder and never its own
//! (brave-core `BraveMainDelegate::PreSandboxStartup` overrides
//! `DIR_USER_NATIVE_MESSAGING` with `Google/Chrome/NativeMessagingHosts`).
//! Opera reads Google Chrome's folder too (it does on Linux, traced, and
//! Opera's developers said so for macOS); its own folder is written as well
//! in case a release reads it.

use std::path::{Path, PathBuf};

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

/// Whether `browser` reads its own folder and whether it reads Google
/// Chrome's.
const fn reads(browser: Browser) -> (bool, bool) {
    match browser {
        Browser::Brave => (false, true),
        Browser::Opera => (true, true),
        _ => (true, false),
    }
}

/// The folder of `browser`'s stable channel, whose existence tells that the
/// browser has run for this user.
pub fn own_root(browser: Browser, home: &Path) -> PathBuf {
    let (_, dir) = support_dirs(browser)[0];
    home.join("Library/Application Support").join(dir)
}

/// `home` must be the real home directory. A sandboxed app has to ask the
/// user database for it, because `$HOME` points into its container.
pub fn targets(browsers: &[Browser], home: &Path) -> Vec<Target> {
    let support = home.join("Library/Application Support");
    let mut targets = Vec::new();
    for &browser in browsers {
        let (own, chromes) = reads(browser);
        for &(label, dir) in support_dirs(browser) {
            let root = support.join(dir);
            if own {
                targets.push(
                    Target::file(label, browser.family(), &root.join("NativeMessagingHosts"))
                        .requiring(&root),
                );
            }
            if chromes {
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
mod tests;
