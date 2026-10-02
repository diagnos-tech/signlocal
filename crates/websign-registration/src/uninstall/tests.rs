use std::path::{Path, PathBuf};

use super::*;
use crate::browsers::Family;
use crate::{linux, macos, system, windows};
use websign_project::NATIVE_HOST;

const CHROME_KEY: &str = r"Software\Google\Chrome";
const CHROMIUM_KEY: &str = r"Software\Chromium";

fn windows_targets(browsers: &[Browser]) -> Vec<Target> {
    windows::targets(browsers, Path::new(r"C:\Users\u\AppData\Local\websign"))
}

fn windows_keys(targets: &[Target]) -> Vec<String> {
    targets
        .iter()
        .filter_map(|target| match &target.location {
            Location::Registry { subkey, .. } => Some(subkey.clone()),
            Location::File { .. } => None,
        })
        .collect()
}

fn host_key(vendor: &str) -> String {
    format!(r"{vendor}\NativeMessagingHosts\{NATIVE_HOST}")
}

fn manifests(targets: &[Target]) -> Vec<PathBuf> {
    targets
        .iter()
        .filter_map(|target| match &target.location {
            Location::File { manifest, .. } => Some(manifest.clone()),
            Location::Registry { .. } => None,
        })
        .collect()
}

fn linux_targets(browsers: &[Browser]) -> Vec<Target> {
    linux::targets(browsers, Path::new("/home/u"), Path::new("/home/u/.config"))
}

fn system_targets(browsers: &[Browser]) -> Vec<Target> {
    system::targets(browsers, &[Path::new("/usr/lib")])
}

#[test]
fn windows_brave_uninstall_keeps_the_shared_chrome_and_chromium_keys() {
    let keys = windows_keys(&exclusive(&[Browser::Brave], windows_targets));
    assert_eq!(keys, [host_key(r"Software\BraveSoftware\Brave-Browser")]);
}

#[test]
fn windows_uninstalling_every_browser_that_reads_a_key_removes_it() {
    let chromium_family = [
        Browser::Chrome,
        Browser::Chromium,
        Browser::Brave,
        Browser::Vivaldi,
        Browser::Opera,
    ];
    let keys = windows_keys(&exclusive(&chromium_family, windows_targets));
    assert!(keys.contains(&host_key(CHROME_KEY)));
    assert!(keys.contains(&host_key(CHROMIUM_KEY)));
}

#[test]
fn windows_chrome_uninstall_keeps_its_key_while_opera_is_registered() {
    let keys = windows_keys(&exclusive(&[Browser::Chrome], windows_targets));
    assert!(keys.is_empty(), "{keys:?}");
}

#[test]
fn windows_chrome_key_stays_while_chromium_still_reads_it() {
    let picked = [
        Browser::Chrome,
        Browser::Brave,
        Browser::Opera,
        Browser::Vivaldi,
    ];
    let keys = windows_keys(&exclusive(&picked, windows_targets));
    assert!(!keys.contains(&host_key(CHROME_KEY)));
}

#[test]
fn windows_chrome_key_goes_once_every_browser_writing_it_goes() {
    let picked = [
        Browser::Chrome,
        Browser::Chromium,
        Browser::Brave,
        Browser::Opera,
        Browser::Vivaldi,
    ];
    let keys = windows_keys(&exclusive(&picked, windows_targets));
    assert!(keys.contains(&host_key(CHROME_KEY)));
}

#[test]
fn windows_keeps_the_shared_chromium_manifest_file_for_remaining_browsers() {
    let remaining = exclusive(&[Browser::Brave], windows_targets);
    assert!(manifests(&remaining).is_empty());
    let own = exclusive(&[Browser::Firefox], windows_targets);
    assert_eq!(manifests(&own).len(), 1);
}

#[test]
fn windows_key_comparison_ignores_case() {
    let lower = Target {
        label: "a".into(),
        family: Family::Chromium,
        location: Location::Registry {
            subkey: r"software\google\chrome".into(),
            manifest: PathBuf::from("m"),
        },
    };
    let upper = Target {
        location: Location::Registry {
            subkey: CHROME_KEY.into(),
            manifest: PathBuf::from("m"),
        },
        ..lower.clone()
    };
    assert!(without_shared(vec![lower], &[upper]).is_empty());
}

#[test]
fn linux_opera_uninstall_keeps_chromes_folder() {
    let removed = exclusive(&[Browser::Opera], linux_targets);
    let chrome = Path::new("/home/u/.config/google-chrome/NativeMessagingHosts");
    assert!(manifests(&removed).iter().all(|m| !m.starts_with(chrome)));
    // Its own Flatpak folder is not shared with anyone.
    assert!(
        manifests(&removed)
            .iter()
            .any(|m| m.starts_with("/home/u/.var/app/com.opera.Opera"))
    );
}

#[test]
fn linux_full_uninstall_removes_chromes_folder() {
    let removed = exclusive(&Browser::ALL, linux_targets);
    assert_eq!(removed.len(), linux_targets(&Browser::ALL).len());
}

#[test]
fn linux_opera_and_chrome_together_remove_chromes_folder() {
    let removed = exclusive(&[Browser::Chrome, Browser::Opera], linux_targets);
    let chrome = Path::new("/home/u/.config/google-chrome/NativeMessagingHosts");
    assert!(manifests(&removed).iter().any(|m| m.starts_with(chrome)));
}

#[test]
fn macos_opera_uninstall_keeps_chromes_folder() {
    let build = |browsers: &[Browser]| macos::targets(browsers, Path::new("/Users/u"));
    let removed = manifests(&exclusive(&[Browser::Opera], build));
    let chrome = Path::new("/Users/u/Library/Application Support/Google/Chrome");
    assert!(!removed.is_empty());
    assert!(removed.iter().all(|m| !m.starts_with(chrome)));
}

#[test]
fn macos_brave_uninstall_keeps_chromes_folder_for_chrome_and_opera() {
    let build = |browsers: &[Browser]| macos::targets(browsers, Path::new("/Users/u"));
    assert!(exclusive(&[Browser::Brave], build).is_empty());
    let removed = manifests(&exclusive(
        &[Browser::Chrome, Browser::Brave, Browser::Opera],
        build,
    ));
    let chrome = Path::new("/Users/u/Library/Application Support/Google/Chrome");
    assert!(removed.iter().any(|m| m.starts_with(chrome)));
}

#[test]
fn system_brave_opera_and_vivaldi_keep_chromes_folder_for_chrome() {
    for browser in [Browser::Brave, Browser::Opera, Browser::Vivaldi] {
        assert!(
            exclusive(&[browser], system_targets).is_empty(),
            "{browser:?}"
        );
    }
}

#[test]
fn system_folder_goes_when_all_its_browsers_go() {
    let picked = [
        Browser::Chrome,
        Browser::Brave,
        Browser::Opera,
        Browser::Vivaldi,
    ];
    let removed = manifests(&exclusive(&picked, system_targets));
    assert_eq!(
        removed,
        [Path::new("/etc/opt/chrome/native-messaging-hosts").join(crate::manifest::file_name())]
    );
}

#[test]
fn unshared_browsers_keep_all_their_targets() {
    let removed = exclusive(&[Browser::Edge], system_targets);
    assert_eq!(removed.len(), 1);
    let removed = exclusive(&[Browser::Firefox], linux_targets);
    assert_eq!(removed.len(), linux_targets(&[Browser::Firefox]).len());
}
