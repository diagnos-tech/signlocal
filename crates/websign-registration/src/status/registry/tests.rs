//! Windows status against an in-memory registry and throwaway manifests.

use std::fs;

use super::*;
use crate::destination::{self, Action, Context};
use crate::registry::MemoryRegistry;
use websign_project::NATIVE_HOST;

/// Absolute on every OS, as a real host path is (a Windows-style path would
/// read as relative elsewhere).
fn host() -> PathBuf {
    std::env::temp_dir().join("websign.exe")
}

fn install(registry: &MemoryRegistry, folder: &Path, browsers: &[Browser]) {
    let host = host();
    let origins = crate::manifest::allowed_origins();
    let context = Context {
        host: &host,
        origins: &origins,
        dry_run: false,
    };
    for target in windows::targets(browsers, folder) {
        destination::apply_with(&target, Action::Install, &context, registry);
    }
}

#[test]
fn no_key_is_missing_and_a_written_key_is_registered() {
    let folder = tempfile::tempdir().unwrap();
    let registry = MemoryRegistry::new();
    assert_eq!(
        state(Browser::Edge, &registry, &host()),
        RegistrationState::Missing
    );
    install(&registry, folder.path(), &Browser::ALL);
    for browser in Browser::ALL {
        assert_eq!(
            state(browser, &registry, &host()),
            RegistrationState::Registered,
            "{browser:?}"
        );
    }
}

#[test]
fn a_key_pointing_to_a_deleted_manifest_is_broken() {
    let folder = tempfile::tempdir().unwrap();
    let registry = MemoryRegistry::new();
    install(&registry, folder.path(), &[Browser::Firefox]);
    for entry in fs::read_dir(folder.path()).unwrap() {
        fs::remove_file(entry.unwrap().path()).unwrap();
    }
    assert!(matches!(
        state(Browser::Firefox, &registry, &host()),
        RegistrationState::Broken { .. }
    ));
}

#[test]
fn chromes_key_is_the_last_fallback_of_chromium_browsers() {
    let folder = tempfile::tempdir().unwrap();
    let registry = MemoryRegistry::new();
    install(&registry, folder.path(), &[Browser::Chrome]);
    assert_eq!(
        state(Browser::Edge, &registry, &host()),
        RegistrationState::Registered
    );
    for browser in [Browser::Brave, Browser::Vivaldi, Browser::Opera] {
        assert_eq!(
            state(browser, &registry, &host()),
            RegistrationState::Registered,
            "{browser:?} reads Chrome's key last"
        );
    }
    assert_eq!(
        state(Browser::Firefox, &registry, &host()),
        RegistrationState::Missing
    );
}

#[test]
fn the_browsers_own_key_shadows_the_fallback() {
    let folder = tempfile::tempdir().unwrap();
    let registry = MemoryRegistry::new();
    install(&registry, folder.path(), &[Browser::Chrome]);
    let stale = folder.path().join("stale.json");
    fs::write(&stale, "not json").unwrap();
    registry
        .set_string(
            &format!(r"Software\Microsoft\Edge\NativeMessagingHosts\{NATIVE_HOST}"),
            "",
            &stale.to_string_lossy(),
        )
        .unwrap();
    assert!(matches!(
        state(Browser::Edge, &registry, &host()),
        RegistrationState::Broken { .. }
    ));
}
