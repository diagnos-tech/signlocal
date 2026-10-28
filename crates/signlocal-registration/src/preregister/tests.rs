use std::path::Path;

use super::*;
use crate::registry::MemoryRegistry;

const CHROME_ID: &str = "abcdefghijklmnopabcdefghijklmnop";
const EDGE_ID: &str = "ponmlkjihgfedcbaponmlkjihgfedcba";

#[test]
fn unpublished_stores_have_nothing_to_preregister() {
    assert!(entries("", "").is_empty());
    assert!(linux::files("", "").is_empty());
    assert!(entries(r"..\..\Policies", "ABCDEFGHIJKLMNOPABCDEFGHIJKLMNOP").is_empty());
    assert!(linux::files("../../etc/x", "").is_empty());
    if CHROME_WEB_STORE_ID.is_empty() && EDGE_ADDONS_ID.is_empty() {
        assert!(apply(false, true).is_empty());
    }
}

#[test]
fn each_store_id_gets_its_browsers_key_and_update_url() {
    let registry = MemoryRegistry::new();
    let results = apply_entries(&registry, &entries(CHROME_ID, EDGE_ID), false, false);
    assert!(
        results.iter().all(|(_, o)| *o == Outcome::Written),
        "{results:?}"
    );
    let url = |subkey: String| {
        registry
            .get_string(Hive::CurrentUser, &subkey, "update_url")
            .unwrap()
    };
    assert_eq!(
        url(format!(r"Software\Google\Chrome\Extensions\{CHROME_ID}")).as_deref(),
        Some(CHROME_UPDATE_URL)
    );
    assert_eq!(
        url(format!(r"Software\Microsoft\Edge\Extensions\{EDGE_ID}")).as_deref(),
        Some(EDGE_UPDATE_URL)
    );
}

#[test]
fn removal_is_idempotent_and_dry_runs_change_nothing() {
    let registry = MemoryRegistry::new();
    let all = entries(CHROME_ID, "");
    assert_eq!(
        apply_entries(&registry, &all, false, true)[0].1,
        Outcome::DryRun
    );
    assert!(registry.current_user_keys().is_empty());
    apply_entries(&registry, &all, false, false);
    assert_eq!(
        apply_entries(&registry, &all, true, true)[0].1,
        Outcome::DryRun
    );
    assert_eq!(
        apply_entries(&registry, &all, true, false)[0].1,
        Outcome::Removed
    );
    assert_eq!(
        apply_entries(&registry, &all, true, false)[0].1,
        Outcome::NotPresent
    );
}

#[test]
fn linux_files_point_the_browsers_at_their_store() {
    let files = linux::files(CHROME_ID, EDGE_ID);
    let paths: Vec<&Path> = files.iter().map(|(path, _)| path.as_path()).collect();
    assert_eq!(
        paths,
        [
            Path::new("/usr/share/google-chrome/extensions/abcdefghijklmnopabcdefghijklmnop.json"),
            Path::new("/usr/share/chromium/extensions/abcdefghijklmnopabcdefghijklmnop.json"),
            Path::new("/usr/share/microsoft-edge/extensions/ponmlkjihgfedcbaponmlkjihgfedcba.json"),
        ]
    );
    let json: serde_json::Value = serde_json::from_str(&files[2].1).unwrap();
    assert_eq!(json["external_update_url"], EDGE_UPDATE_URL);
}
