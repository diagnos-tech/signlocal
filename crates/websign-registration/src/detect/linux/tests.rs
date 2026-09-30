//! Detection over a throwaway root laid out like a Linux system.

use std::fs;

use super::*;

fn executable(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, contents).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn found(root: &Path, apps: &[&str]) -> Vec<(Browser, BrowserPackaging)> {
    let dirs = [root.join("usr/bin"), root.join("snap/bin")];
    let apps: Vec<String> = apps.iter().map(|app| (*app).to_owned()).collect();
    detect(root, &dirs, &apps)
        .into_iter()
        .map(|entry| (entry.browser, entry.packaging))
        .collect()
}

#[test]
fn an_empty_system_has_no_browsers() {
    let root = tempfile::tempdir().unwrap();
    assert!(found(root.path(), &[]).is_empty());
}

#[test]
fn path_opt_snap_and_flatpak_are_each_reported_in_browser_order() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    executable(root, "usr/bin/firefox-esr", "\x7fELF binary");
    executable(root, "opt/microsoft/msedge/msedge", "\x7fELF binary");
    executable(root, "snap/bin/chromium", "\x7fELF binary");
    assert_eq!(
        found(root, &["com.brave.Browser", "org.example.Other"]),
        [
            (Browser::Chromium, BrowserPackaging::Snap),
            (Browser::Edge, BrowserPackaging::Native),
            (Browser::Brave, BrowserPackaging::Flatpak),
            (Browser::Firefox, BrowserPackaging::Native),
        ]
    );
}

#[test]
fn ubuntus_snap_forwarding_script_is_not_a_native_browser() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    executable(
        root,
        "usr/bin/firefox",
        "#!/bin/sh\nexec /snap/bin/firefox \"$@\"\n",
    );
    executable(root, "snap/bin/firefox", "\x7fELF binary");
    assert_eq!(
        found(root, &[]),
        [(Browser::Firefox, BrowserPackaging::Snap)]
    );
}

#[cfg(unix)]
#[test]
fn a_file_without_the_executable_bit_does_not_count() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("usr/bin/vivaldi");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "data").unwrap();
    assert!(found(root.path(), &[]).is_empty());
}
