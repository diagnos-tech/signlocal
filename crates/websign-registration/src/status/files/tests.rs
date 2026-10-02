//! Status read back from a throwaway home after real registrations.

use std::fs;
use std::path::PathBuf;

use super::*;
use crate::browsers::Browser;
use crate::destination::{self, Action, Context};
use crate::linux;
use crate::status::file_candidates;

const HOST: &str = "/usr/bin/websign";

fn register(home: &Path, browsers: &[Browser], host: &str) {
    let host = PathBuf::from(host);
    let origins = crate::manifest::allowed_origins();
    let context = Context {
        host: &host,
        origins: &origins,
        dry_run: false,
    };
    for target in linux::targets(browsers, home, &home.join(".config")) {
        destination::apply(&target, Action::Install, &context);
    }
}

/// The Linux per-user manifests, on every OS: the fixtures lay out a Linux
/// home (`.config`, `.var/app`), and `register` writes that layout, so the
/// platform's own candidates (`~/Library` on macOS) would never see them.
fn user_candidates(browser: Browser, home: &Path) -> Vec<Candidate> {
    file_candidates(linux::targets(&[browser], home, &home.join(".config")))
}

#[test]
fn an_absent_browser_folder_reads_as_missing() {
    let home = tempfile::tempdir().unwrap();
    let state = state(
        &user_candidates(Browser::Vivaldi, home.path()),
        Path::new(HOST),
    );
    assert_eq!(state, RegistrationState::Missing);
}

#[test]
fn a_registered_browser_reads_back_as_registered() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".config/google-chrome")).unwrap();
    fs::create_dir_all(home.path().join(".mozilla")).unwrap();
    register(home.path(), &Browser::ALL, HOST);
    for browser in [Browser::Chrome, Browser::Firefox] {
        let found = state(&user_candidates(browser, home.path()), Path::new(HOST));
        assert_eq!(found, RegistrationState::Registered, "{browser:?}");
    }
}

#[test]
fn a_manifest_from_another_install_points_elsewhere() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".config/chromium")).unwrap();
    register(home.path(), &[Browser::Chromium], "/opt/old/websign");
    assert_eq!(
        state(
            &user_candidates(Browser::Chromium, home.path()),
            Path::new(HOST)
        ),
        RegistrationState::PointsElsewhere {
            path: "/opt/old/websign".into()
        }
    );
}

#[test]
fn a_flatpak_manifest_must_point_at_its_host_copy() {
    let home = tempfile::tempdir().unwrap();
    let real_host = home.path().join("websign");
    fs::write(&real_host, b"binary").unwrap();
    fs::create_dir_all(home.path().join(".var/app/org.chromium.Chromium")).unwrap();
    register(
        home.path(),
        &[Browser::Chromium],
        real_host.to_str().unwrap(),
    );
    assert_eq!(
        state(&user_candidates(Browser::Chromium, home.path()), &real_host),
        RegistrationState::Registered
    );
}

#[test]
fn a_corrupt_manifest_is_broken_unless_another_channel_is_fine() {
    let home = tempfile::tempdir().unwrap();
    for dir in ["google-chrome", "google-chrome-beta"] {
        fs::create_dir_all(home.path().join(".config").join(dir)).unwrap();
    }
    register(home.path(), &[Browser::Chrome], HOST);
    let stable = home
        .path()
        .join(".config/google-chrome/NativeMessagingHosts")
        .join(crate::manifest::file_name());
    fs::write(&stable, "{").unwrap();
    let candidates = user_candidates(Browser::Chrome, home.path());
    assert_eq!(
        state(&candidates, Path::new(HOST)),
        RegistrationState::Registered
    );

    let stable_only: Vec<Candidate> = candidates
        .into_iter()
        .filter(|c| c.manifest == stable)
        .collect();
    assert!(matches!(
        state(&stable_only, Path::new(HOST)),
        RegistrationState::Broken { .. }
    ));
}
