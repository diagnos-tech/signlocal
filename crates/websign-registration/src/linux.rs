//! Where Linux browsers look for user-level native messaging manifests.
//!
//! Chromium-based browsers read `<config dir>/NativeMessagingHosts/`; Firefox
//! reads `~/.mozilla/native-messaging-hosts/`. Snap Firefox reads the same
//! folder, because the portal that starts hosts on its behalf runs outside
//! the sandbox.

use std::path::Path;

use super::browsers::{Browser, Family};
use super::destination::Target;
use websign_project::SLUG;

/// `(label, folder under ~/.config)` for each channel of a Chromium browser.
fn config_dirs(browser: Browser) -> &'static [(&'static str, &'static str)] {
    match browser {
        Browser::Chrome => &[
            ("Google Chrome", "google-chrome"),
            ("Google Chrome Beta", "google-chrome-beta"),
            ("Google Chrome Dev", "google-chrome-unstable"),
            ("Google Chrome for Testing", "google-chrome-for-testing"),
        ],
        Browser::Chromium => &[("Chromium", "chromium")],
        Browser::Edge => &[
            ("Microsoft Edge", "microsoft-edge"),
            ("Microsoft Edge Beta", "microsoft-edge-beta"),
            ("Microsoft Edge Dev", "microsoft-edge-dev"),
        ],
        Browser::Brave => &[
            ("Brave", "BraveSoftware/Brave-Browser"),
            ("Brave Beta", "BraveSoftware/Brave-Browser-Beta"),
            ("Brave Nightly", "BraveSoftware/Brave-Browser-Nightly"),
        ],
        Browser::Vivaldi => &[
            ("Vivaldi", "vivaldi"),
            ("Vivaldi Snapshot", "vivaldi-snapshot"),
        ],
        Browser::Opera => &[
            ("Opera", "opera"),
            ("Opera Beta", "opera-beta"),
            ("Opera Developer", "opera-developer"),
        ],
        Browser::Firefox => &[],
    }
}

/// `(Flatpak app ID, config folder inside it)`; Firefox keeps `.mozilla`.
fn flatpak(browser: Browser) -> Option<(&'static str, &'static str)> {
    Some(match browser {
        Browser::Chrome => ("com.google.Chrome", "google-chrome"),
        Browser::Chromium => ("org.chromium.Chromium", "chromium"),
        Browser::Edge => ("com.microsoft.Edge", "microsoft-edge"),
        Browser::Brave => ("com.brave.Browser", "BraveSoftware/Brave-Browser"),
        Browser::Vivaldi => ("com.vivaldi.Vivaldi", "vivaldi"),
        Browser::Opera => ("com.opera.Opera", "opera"),
        Browser::Firefox => ("org.mozilla.firefox", ".mozilla"),
    })
}

pub fn targets(browsers: &[Browser], home: &Path) -> Vec<Target> {
    let mut targets = Vec::new();
    for &browser in browsers {
        match browser.family() {
            Family::Firefox => {
                let root = home.join(".mozilla");
                targets.push(
                    Target::file(
                        "Firefox (also Snap, through the portal)",
                        Family::Firefox,
                        &root.join("native-messaging-hosts"),
                    )
                    .requiring(&root),
                );
            }
            Family::Chromium => {
                for (label, dir) in config_dirs(browser) {
                    let root = home.join(".config").join(dir);
                    targets.push(
                        Target::file(*label, Family::Chromium, &root.join("NativeMessagingHosts"))
                            .requiring(&root),
                    );
                }
            }
        }
        if browser == Browser::Chromium {
            let root = home.join("snap/chromium/common/chromium");
            targets.push(
                Target::file(
                    "Chromium (Snap)",
                    Family::Chromium,
                    &root.join("NativeMessagingHosts"),
                )
                .requiring(&root),
            );
        }
        targets.extend(flatpak_target(browser, home));
    }
    targets
}

/// Flatpak browsers cannot run the host installed outside their sandbox, so
/// the manifest points to a copy of the binary kept inside the app's own data.
fn flatpak_target(browser: Browser, home: &Path) -> Option<Target> {
    let (app_id, config) = flatpak(browser)?;
    let app = home.join(".var/app").join(app_id);
    let hosts_dir = match browser.family() {
        Family::Firefox => app.join(config).join("native-messaging-hosts"),
        Family::Chromium => app.join("config").join(config).join("NativeMessagingHosts"),
    };
    let copy = hosts_dir.join(format!("{SLUG}-probe"));
    Some(
        Target::file(
            format!("{} (Flatpak)", browser.label()),
            browser.family(),
            &hosts_dir,
        )
        .requiring(&app)
        .with_host_copy(copy),
    )
}

#[cfg(test)]
mod tests;
