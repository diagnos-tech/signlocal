use super::*;

const ID: &str = "nhnkdpljdgjflbflkhnkmfmcmodboeii";

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn chromium_on_linux_and_macos_passes_only_the_origin() {
    let launch = parse_launch(&args(&[&format!("chrome-extension://{ID}/")])).unwrap();
    assert_eq!(launch.family, BrowserFamily::Chromium);
    assert_eq!(launch.extension_id, ID);
    assert_eq!(launch.origin, format!("chrome-extension://{ID}/"));
    assert_eq!(launch.parent_window, None);
}

#[test]
fn chromium_on_windows_adds_the_parent_window() {
    let launch = parse_launch(&args(&[
        &format!("chrome-extension://{ID}/"),
        "--parent-window=1443034",
    ]))
    .unwrap();
    assert_eq!(launch.parent_window, Some(1_443_034));
}

#[test]
fn a_zero_parent_window_means_service_worker_and_is_dropped() {
    let launch = parse_launch(&args(&[
        &format!("chrome-extension://{ID}/"),
        "--parent-window=0",
    ]))
    .unwrap();
    assert_eq!(launch.parent_window, None);
}

#[test]
fn garbage_parent_window_is_ignored_not_fatal() {
    let launch = parse_launch(&args(&[
        &format!("chrome-extension://{ID}/"),
        "--parent-window=abc",
    ]))
    .unwrap();
    assert_eq!(launch.parent_window, None);
}

#[test]
fn origin_without_trailing_slash_is_accepted() {
    let launch = parse_launch(&args(&[&format!("chrome-extension://{ID}")])).unwrap();
    assert_eq!(launch.extension_id, ID);
}

#[test]
fn malformed_chromium_ids_are_not_launches() {
    for bad in [
        "chrome-extension:///",
        "chrome-extension://short/",
        "chrome-extension://NHNKDPLJDGJFLBFLKHNKMFMCMODBOEII/",
        "chrome-extension://nhnkdpljdgjflbflkhnkmfmcmodboeiz/",
        &format!("chrome-extension://{ID}/extra/"),
    ] {
        assert_eq!(parse_launch(&args(&[bad])), None, "{bad}");
    }
}

#[test]
fn firefox_passes_manifest_path_then_extension_id() {
    let launch = parse_launch(&args(&[
        "/home/u/.mozilla/native-messaging-hosts/dev.websign.host.json",
        "websign@dev.websign",
    ]))
    .unwrap();
    assert_eq!(launch.family, BrowserFamily::Firefox);
    assert_eq!(launch.extension_id, "websign@dev.websign");
    assert_eq!(launch.origin, "websign@dev.websign");
}

#[test]
fn firefox_on_windows_passes_a_backslash_path() {
    let launch = parse_launch(&args(&[
        r"C:\Users\u\AppData\Local\websign\dev.websign.host.firefox.json",
        "{e68418bc-f2b0-4459-a9ea-3e72b6751b07}",
    ]))
    .unwrap();
    assert_eq!(launch.family, BrowserFamily::Firefox);
}

#[test]
fn cli_invocations_are_never_mistaken_for_launches() {
    for cli in [
        &[][..],
        &["list"],
        &["sign", "--all"],
        &["register", "--browser", "firefox"],
        &["host"],
        &["--version"],
        &["help", "list"],
        // A lone manifest path is not enough: Firefox always adds the ID.
        &["/tmp/x.json"],
        &["/tmp/x.json", "--flag"],
        &["notes.json", "id@x"],
    ] {
        assert_eq!(parse_launch(&args(cli)), None, "{cli:?}");
    }
}
