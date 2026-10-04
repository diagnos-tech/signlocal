//! The Windows plan applied end to end against an in-memory registry and a
//! throwaway manifest folder.

use std::fs;

use super::*;
use crate::browsers::Browser;
use crate::registry::{Hive, MemoryRegistry};
use crate::windows;
use websign_project::NATIVE_HOST;

fn apply_all(targets: &[Target], action: Action, registry: &MemoryRegistry) -> Vec<Outcome> {
    let host = PathBuf::from(r"C:\Programs\SignLocal\websign.exe");
    let context = Context {
        host: &host,
        origins: &[],
        dry_run: false,
    };
    targets
        .iter()
        .map(|target| apply_with(target, action, &context, registry))
        .collect()
}

#[test]
fn install_points_every_key_at_an_existing_manifest_and_uninstall_is_idempotent() {
    let folder = tempfile::tempdir().unwrap();
    let registry = MemoryRegistry::new();
    let targets = windows::targets(&Browser::ALL, folder.path());

    let installed = apply_all(&targets, Action::Install, &registry);
    assert!(
        installed.iter().all(|o| *o == Outcome::Written),
        "{installed:?}"
    );
    for target in &targets {
        if let Location::Registry { subkey, manifest } = &target.location {
            let value = registry
                .get_string(Hive::CurrentUser, subkey, "")
                .unwrap()
                .unwrap();
            assert_eq!(Path::new(&value), manifest);
            assert!(fs::metadata(manifest).is_ok(), "{value} must exist");
        }
    }

    let removed = apply_all(&targets, Action::Uninstall, &registry);
    assert!(
        removed.iter().all(|o| *o == Outcome::Removed),
        "{removed:?}"
    );
    let ours = format!("nativemessaginghosts\\{}", NATIVE_HOST.to_lowercase());
    assert!(
        registry
            .current_user_keys()
            .iter()
            .all(|key| !key.contains(&ours))
    );
    let again = apply_all(&targets, Action::Uninstall, &registry);
    assert!(again.iter().all(|o| *o == Outcome::NotPresent), "{again:?}");
}

#[test]
fn registry_dry_run_writes_nothing() {
    let registry = MemoryRegistry::new();
    let target = Target {
        label: "Edge".into(),
        family: Family::Chromium,
        location: Location::Registry {
            subkey: r"Software\Microsoft\Edge\NativeMessagingHosts\x".into(),
            manifest: PathBuf::from(r"C:\m\x.json"),
        },
    };
    let host = PathBuf::from(r"C:\w.exe");
    let context = Context {
        host: &host,
        origins: &[],
        dry_run: true,
    };
    assert_eq!(
        apply_with(&target, Action::Install, &context, &registry),
        Outcome::DryRun
    );
    assert!(registry.current_user_keys().is_empty());
}
