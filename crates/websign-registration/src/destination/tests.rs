use std::fs;
use std::sync::atomic::{AtomicU32, Ordering};

use serde_json::Value;

use super::*;
use websign_project::{FIREFOX_ID, NATIVE_HOST};

/// A scratch folder removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "websign-register-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn context<'a>(host: &'a Path, origins: &'a [String], dry_run: bool) -> Context<'a> {
    Context {
        host,
        origins,
        dry_run,
    }
}

fn read_manifest(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn install_writes_the_manifest_and_creates_missing_folders() {
    let scratch = Scratch::new();
    let hosts = scratch.path("profile/NativeMessagingHosts");
    let target = Target::file("Test", Family::Chromium, &hosts);
    let origins = vec!["chrome-extension://abcdefghijklmnopabcdefghijklmnop/".to_owned()];
    let host = PathBuf::from("/opt/websign/probe");

    let outcome = apply(&target, Action::Install, &context(&host, &origins, false));

    assert_eq!(outcome, Outcome::Written);
    let manifest = read_manifest(&hosts.join(format!("{NATIVE_HOST}.json")));
    assert_eq!(manifest["name"], NATIVE_HOST);
    assert_eq!(manifest["path"], "/opt/websign/probe");
    assert_eq!(manifest["allowed_origins"][0], origins[0]);
}

#[test]
fn firefox_manifest_carries_the_gecko_id() {
    let scratch = Scratch::new();
    let target = Target::file("Test", Family::Firefox, &scratch.path("hosts"));
    let host = PathBuf::from("/opt/websign/probe");
    apply(&target, Action::Install, &context(&host, &[], false));
    let manifest = read_manifest(&scratch.path(&format!("hosts/{NATIVE_HOST}.json")));
    assert_eq!(manifest["allowed_extensions"][0], FIREFOX_ID);
}

#[test]
fn a_missing_browser_folder_skips_without_creating_anything() {
    let scratch = Scratch::new();
    let root = scratch.path("vivaldi");
    let target = Target::file(
        "Vivaldi",
        Family::Chromium,
        &root.join("NativeMessagingHosts"),
    )
    .requiring(&root);
    let host = PathBuf::from("/opt/websign/probe");

    let outcome = apply(&target, Action::Install, &context(&host, &[], false));

    assert!(matches!(outcome, Outcome::Skipped(_)), "{outcome:?}");
    assert!(!root.exists(), "must not create another vendor's folder");
}

#[test]
fn an_existing_browser_folder_is_written_into() {
    let scratch = Scratch::new();
    let root = scratch.path("chrome");
    fs::create_dir_all(&root).unwrap();
    let target = Target::file(
        "Chrome",
        Family::Chromium,
        &root.join("NativeMessagingHosts"),
    )
    .requiring(&root);
    let host = PathBuf::from("/opt/websign/probe");
    assert_eq!(
        apply(&target, Action::Install, &context(&host, &[], false)),
        Outcome::Written
    );
}

#[test]
fn dry_run_changes_nothing() {
    let scratch = Scratch::new();
    let hosts = scratch.path("hosts");
    let target = Target::file("Test", Family::Chromium, &hosts);
    let host = PathBuf::from("/opt/websign/probe");
    let outcome = apply(&target, Action::Install, &context(&host, &[], true));
    assert_eq!(outcome, Outcome::DryRun);
    assert!(!hosts.exists());
}

#[test]
fn uninstall_removes_the_manifest_and_reports_when_it_is_gone() {
    let scratch = Scratch::new();
    let hosts = scratch.path("hosts");
    let target = Target::file("Test", Family::Chromium, &hosts);
    let host = PathBuf::from("/opt/websign/probe");
    let ctx = context(&host, &[], false);
    apply(&target, Action::Install, &ctx);

    assert_eq!(apply(&target, Action::Uninstall, &ctx), Outcome::Removed);
    assert!(!hosts.join(format!("{NATIVE_HOST}.json")).exists());
    assert_eq!(apply(&target, Action::Uninstall, &ctx), Outcome::NotPresent);
}

#[test]
fn uninstall_dry_run_keeps_the_manifest() {
    let scratch = Scratch::new();
    let hosts = scratch.path("hosts");
    let target = Target::file("Test", Family::Chromium, &hosts);
    let host = PathBuf::from("/opt/websign/probe");
    apply(&target, Action::Install, &context(&host, &[], false));
    assert_eq!(
        apply(&target, Action::Uninstall, &context(&host, &[], true)),
        Outcome::DryRun
    );
    assert!(hosts.join(format!("{NATIVE_HOST}.json")).exists());
}

#[test]
fn flatpak_style_targets_copy_the_host_and_point_the_manifest_at_the_copy() {
    let scratch = Scratch::new();
    let real_host = scratch.path("real-host");
    fs::write(&real_host, b"binary").unwrap();
    let hosts = scratch.path("app/hosts");
    let copy = hosts.join("websign-probe");
    let target = Target::file("Flatpak", Family::Chromium, &hosts).with_host_copy(copy.clone());
    let ctx = context(&real_host, &[], false);

    assert_eq!(apply(&target, Action::Install, &ctx), Outcome::Written);
    assert_eq!(fs::read(&copy).unwrap(), b"binary");
    let manifest = read_manifest(&hosts.join(format!("{NATIVE_HOST}.json")));
    assert_eq!(manifest["path"], copy.to_str().unwrap());

    assert_eq!(apply(&target, Action::Uninstall, &ctx), Outcome::Removed);
    assert!(!copy.exists(), "the copy goes away with the manifest");
}

#[cfg(unix)]
#[test]
fn a_folder_that_cannot_be_written_is_reported_as_failed() {
    let scratch = Scratch::new();
    let blocker = scratch.path("file-not-folder");
    fs::write(&blocker, b"x").unwrap();
    let target = Target::file(
        "Test",
        Family::Chromium,
        &blocker.join("NativeMessagingHosts"),
    );
    let host = PathBuf::from("/opt/websign/probe");
    let outcome = apply(&target, Action::Install, &context(&host, &[], false));
    assert!(matches!(outcome, Outcome::Failed(_)), "{outcome:?}");
}
