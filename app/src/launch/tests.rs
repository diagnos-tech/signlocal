//! Every argv shape the app is started with.

use super::*;

/// Our development extension's ID (`project.toml`).
const ID: &str = websign_project::EXTENSION_DEV_ID;
/// A well-formed ID of somebody else's extension.
const FOREIGN_ID: &str = "abcdefghijklmnopabcdefghijklmnop";
const FIREFOX_ID: &str = websign_project::FIREFOX_ID;
const SAFARI_ID: &str = websign_project::SAFARI_EXTENSION_BUNDLE_ID;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

fn browser(list: &[&str]) -> BrowserLaunch {
    browser_at(false, list)
}

fn browser_at(in_appex: bool, list: &[&str]) -> BrowserLaunch {
    match classify(in_appex, &args(list)) {
        Launch::Browser(launch) => launch,
        other => panic!("{list:?} → {other:?}"),
    }
}

#[test]
fn chrome_and_edge_on_linux_and_macos_pass_the_origin_alone() {
    let launch = browser(&[&format!("chrome-extension://{ID}/")]);
    assert_eq!(launch.family, BrowserFamily::Chromium);
    assert_eq!(launch.extension_id, ID);
    assert_eq!(launch.parent_window, None);
}

#[test]
fn chrome_and_edge_on_windows_add_the_parent_window() {
    let launch = browser(&[
        &format!("chrome-extension://{ID}/"),
        "--parent-window=132456",
    ]);
    assert_eq!(launch.parent_window, Some(132456));
    // A service worker caller reports zero: no owner window.
    let launch = browser(&[&format!("chrome-extension://{ID}/"), "--parent-window=0"]);
    assert_eq!(launch.parent_window, None);
}

#[test]
fn firefox_passes_the_manifest_path_and_the_extension_id() {
    for manifest in [
        "/home/u/.mozilla/native-messaging-hosts/dev.websign.host.json",
        r"C:\Users\u\AppData\Local\websign\NativeMessagingHosts\dev.websign.host.json",
        "/Users/u/Library/Application Support/Mozilla/NativeMessagingHosts/dev.websign.host.json",
    ] {
        let launch = browser(&[manifest, FIREFOX_ID]);
        assert_eq!(launch.family, BrowserFamily::Firefox);
        assert_eq!(launch.extension_id, FIREFOX_ID);
    }
}

#[test]
fn the_safari_app_extension_passes_its_flag_and_bundle_id() {
    let launch = browser_at(true, &[websign_host::SAFARI_FLAG, SAFARI_ID]);
    assert_eq!(launch.family, BrowserFamily::Safari);
    assert_eq!(launch.extension_id, SAFARI_ID);
}

#[test]
fn the_safari_shape_outside_the_appex_is_a_command_line() {
    let list = args(&[websign_host::SAFARI_FLAG, SAFARI_ID]);
    assert_eq!(classify(false, &list), Launch::Command);
}

#[test]
fn another_bundle_id_never_starts_host_mode_from_the_appex() {
    let list = args(&[websign_host::SAFARI_FLAG, "org.example.other.extension"]);
    assert_eq!(classify(true, &list), Launch::Command);
    // Chromium and Firefox launches still work from the appex's copy.
    let launch = browser_at(true, &[&format!("chrome-extension://{ID}/")]);
    assert_eq!(launch.family, BrowserFamily::Chromium);
}

#[test]
fn other_extensions_never_start_host_mode() {
    for list in [
        vec![format!("chrome-extension://{FOREIGN_ID}/")],
        vec![
            format!("chrome-extension://{FOREIGN_ID}/"),
            "--parent-window=1".to_owned(),
        ],
        vec![
            "/home/u/.mozilla/native-messaging-hosts/dev.websign.host.json".to_owned(),
            "other@example.org".to_owned(),
        ],
        vec!["./notes.json".to_owned(), "id".to_owned()],
    ] {
        assert_eq!(classify(false, &list), Launch::Command, "{list:?}");
    }
}

#[test]
fn connect_and_no_arguments_are_their_own_launches() {
    assert_eq!(classify(false, &args(&["connect"])), Launch::Connect);
    assert_eq!(classify(false, &[]), Launch::Gui);
}

#[test]
fn websign_urls_name_only_the_documented_actions() {
    for url in [
        "websign:activate",
        "WEBSIGN:Activate",
        "websign://activate/",
        "websign:activate/",
    ] {
        assert_eq!(
            classify(false, &args(&[url])),
            Launch::Url(UrlAction::Activate),
            "{url}"
        );
    }
    for url in [
        "websign:",
        "websign:sign?digest=00",
        "websign://activate?x=1",
        "websign:activate/../../etc",
        &format!("websign:{}", "a".repeat(300)),
    ] {
        assert_eq!(
            classify(false, &args(&[url])),
            Launch::Url(UrlAction::Diagnostics),
            "{url}"
        );
    }
}

#[test]
fn everything_else_is_a_command_line() {
    for list in [
        &["sign", "--hash", "SHA-256"][..],
        &["connect", "--verbose"],
        &["version"],
        &["websignactivate"],
        &["chrome-extension://short/"],
        &["notes.json", "id"],
        &["--help"],
        &["web:activate"],
        &["websign:activate", "extra"],
    ] {
        assert_eq!(classify(false, &args(list)), Launch::Command, "{list:?}");
    }
}

#[test]
fn launch_services_serials_are_dropped_on_macos_only() {
    assert_eq!(
        is_launch_services_serial("-psn_0_1234567"),
        cfg!(target_os = "macos")
    );
    assert!(!is_launch_services_serial("--help"));
    assert!(!is_launch_services_serial("psn_0_1"));
}
