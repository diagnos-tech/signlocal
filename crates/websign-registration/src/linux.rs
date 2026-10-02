//! Where Linux browsers look for user-level native messaging manifests.
//!
//! Chromium-based browsers read `<config dir>/NativeMessagingHosts/` under
//! `$XDG_CONFIG_HOME` (default `~/.config`); Firefox reads
//! `~/.mozilla/native-messaging-hosts/`. Opera reads Google Chrome's folder
//! instead of its own, and Brave reads its default folder even when started
//! with another `--user-data-dir` (both traced,
//! `docs/research/native-messaging.md` §3.6). Snap Firefox reads the same
//! folder (or the system one), because the WebExtensions portal that starts
//! hosts on its behalf runs outside the sandbox; the portal asks the user once
//! per extension. Flatpak browsers see only their own `~/.var/app/<id>`, so
//! they get a manifest and a copy of the host there; that copy answers but
//! cannot reach `pcscd` or system PKCS#11 modules from inside the sandbox.

use std::path::{Path, PathBuf};

use super::browsers::{Browser, Family};
use super::destination::Target;
use websign_project::SLUG;

/// `(label, folder under the config home)` for each channel of a Chromium
/// browser: the folder that must exist for the browser to count as installed.
fn config_dirs(browser: Browser) -> &'static [(&'static str, &'static str)] {
    match browser {
        Browser::Chrome => &[
            ("Google Chrome", "google-chrome"),
            ("Google Chrome Beta", "google-chrome-beta"),
            ("Google Chrome Dev", "google-chrome-unstable"),
            ("Google Chrome Canary", "google-chrome-canary"),
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

/// The folder under the config home whose `NativeMessagingHosts` the browser
/// reads, when it is not the browser's own: every Opera channel reads Google
/// Chrome's.
fn hosts_owner(browser: Browser) -> Option<&'static str> {
    (browser == Browser::Opera).then_some("google-chrome")
}

/// The folder of `browser`'s stable channel, whose existence tells that the
/// browser has run for this user: `~/.mozilla` for Firefox, else its folder
/// under `config`.
pub fn own_root(browser: Browser, home: &Path, config: &Path) -> PathBuf {
    match config_dirs(browser).first() {
        Some((_, dir)) => config.join(dir),
        None => home.join(".mozilla"),
    }
}

/// `$XDG_CONFIG_HOME` when it is absolute (the XDG rule, which Chromium
/// follows), else `<home>/.config`.
pub fn config_home(home: &Path) -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .unwrap_or_else(|| home.join(".config"))
}

/// `(Flatpak app ID, config folder inside it)`; Firefox keeps `.mozilla`.
pub(crate) fn flatpak(browser: Browser) -> Option<(&'static str, &'static str)> {
    Some(match browser {
        Browser::Chrome => ("com.google.Chrome", "google-chrome"),
        Browser::Chromium => ("org.chromium.Chromium", "chromium"),
        Browser::Edge => ("com.microsoft.Edge", "microsoft-edge"),
        Browser::Brave => ("com.brave.Browser", "BraveSoftware/Brave-Browser"),
        Browser::Vivaldi => ("com.vivaldi.Vivaldi", "vivaldi"),
        Browser::Opera => ("com.opera.Opera", "google-chrome"),
        Browser::Firefox => ("org.mozilla.firefox", ".mozilla"),
    })
}

/// Per-user targets; `config` is [`config_home`] (a parameter so tests need
/// no environment).
pub fn targets(browsers: &[Browser], home: &Path, config: &Path) -> Vec<Target> {
    let mut targets = Vec::new();
    for &browser in browsers {
        match browser.family() {
            Family::Firefox => {
                let root = home.join(".mozilla");
                let hosts = root.join("native-messaging-hosts");
                targets.push(Target::file("Firefox", Family::Firefox, &hosts).requiring(&root));
                // The Snap keeps its profile in `~/snap/firefox`, so
                // `~/.mozilla` may not exist; the portal still reads it.
                targets.push(
                    Target::file(
                        "Firefox (Snap, through the portal)",
                        Family::Firefox,
                        &hosts,
                    )
                    .requiring(&home.join("snap/firefox")),
                );
            }
            Family::Chromium => {
                for (label, dir) in config_dirs(browser) {
                    let root = config.join(dir);
                    let hosts = config.join(hosts_owner(browser).unwrap_or(dir));
                    targets.push(
                        Target::file(
                            *label,
                            Family::Chromium,
                            &hosts.join("NativeMessagingHosts"),
                        )
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
    let copy = hosts_dir.join(SLUG);
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
