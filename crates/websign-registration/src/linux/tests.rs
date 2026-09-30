use std::path::PathBuf;

use super::*;
use crate::destination::Location;
use crate::manifest;

fn manifests(browsers: &[Browser]) -> Vec<(String, PathBuf, Option<PathBuf>)> {
    targets(browsers, Path::new("/home/u"))
        .into_iter()
        .map(|target| {
            let Location::File {
                manifest, requires, ..
            } = target.location
            else {
                panic!("Linux registers files only");
            };
            (target.label, manifest, requires)
        })
        .collect()
}

fn manifest_for(label: &str, browsers: &[Browser]) -> PathBuf {
    manifests(browsers)
        .into_iter()
        .find(|(l, _, _)| l == label)
        .unwrap_or_else(|| panic!("no target labelled {label}"))
        .1
}

#[test]
fn chrome_and_chromium_use_their_config_folders() {
    let name = manifest::file_name();
    assert_eq!(
        manifest_for("Google Chrome", &[Browser::Chrome]),
        PathBuf::from("/home/u/.config/google-chrome/NativeMessagingHosts").join(&name)
    );
    assert_eq!(
        manifest_for("Chromium", &[Browser::Chromium]),
        PathBuf::from("/home/u/.config/chromium/NativeMessagingHosts").join(&name)
    );
    assert_eq!(
        manifest_for("Brave", &[Browser::Brave]),
        PathBuf::from("/home/u/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts")
            .join(&name)
    );
}

#[test]
fn firefox_uses_the_lowercase_hyphenated_folder() {
    let (_, manifest, requires) = manifests(&[Browser::Firefox]).remove(0);
    assert_eq!(
        manifest,
        PathBuf::from("/home/u/.mozilla/native-messaging-hosts").join(manifest::file_name())
    );
    assert_eq!(requires, Some(PathBuf::from("/home/u/.mozilla")));
}

#[test]
fn a_browser_counts_as_installed_when_its_config_root_exists() {
    for (label, manifest, requires) in manifests(&Browser::ALL) {
        let requires = requires.unwrap_or_else(|| panic!("{label} must require its root"));
        assert!(manifest.starts_with(&requires), "{label}: {manifest:?}");
    }
}

#[test]
fn snap_chromium_and_flatpak_browsers_have_their_own_targets() {
    let all = manifests(&Browser::ALL);
    assert!(all.iter().any(|(label, path, _)| label == "Chromium (Snap)"
        && path.starts_with("/home/u/snap/chromium/common/chromium")));
    assert!(all.iter().any(|(label, path, _)| {
        label == "Firefox (Flatpak)"
            && path
                .starts_with("/home/u/.var/app/org.mozilla.firefox/.mozilla/native-messaging-hosts")
    }));
    assert!(
        all.iter()
            .any(|(label, path, _)| label == "Google Chrome (Flatpak)"
                && path.starts_with(
                    "/home/u/.var/app/com.google.Chrome/config/google-chrome/NativeMessagingHosts"
                ))
    );
}

#[test]
fn flatpak_targets_copy_the_host_next_to_the_manifest() {
    let flatpak = targets(&[Browser::Chrome], Path::new("/home/u"))
        .into_iter()
        .find(|t| t.label == "Google Chrome (Flatpak)")
        .unwrap();
    let Location::File {
        manifest,
        host_copy,
        ..
    } = flatpak.location
    else {
        panic!("expected a file target");
    };
    assert_eq!(host_copy.unwrap().parent(), manifest.parent());
}
