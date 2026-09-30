use super::*;
use crate::testutil::TempDir;

fn repo(cargo: &str, sdk: &str, min: &str) -> TempDir {
    let dir = TempDir::new();
    dir.write(
        "Cargo.toml",
        &format!("[workspace.package]\nversion = \"{cargo}\"\n"),
    );
    dir.write(
        "project.toml",
        &format!("[extension]\nmin_app_version = \"{min}\"\n"),
    );
    dir.write("sdk/package.json", &format!("{{\"version\": \"{sdk}\"}}"));
    dir.write(
        "clients/node/package.json",
        &format!("{{\"version\": \"{cargo}\"}}"),
    );
    dir.write(
        "extension/package.json",
        &format!("{{\"version\": \"{cargo}\"}}"),
    );
    dir
}

#[test]
fn matching_versions_pass() {
    assert!(
        check(repo("1.2.3", "1.2.3", "1.0.0").path(), Some("v1.2.3"))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_package_out_of_lockstep_is_named() {
    let found = check(repo("1.2.3", "1.2.4", "1.0.0").path(), None).unwrap();
    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("sdk/package.json"), "{found:?}");
}

#[test]
fn a_min_app_version_above_the_release_is_refused() {
    let found = check(repo("0.1.0", "0.1.0", "0.2.0").path(), None).unwrap();
    assert!(found[0].contains("min_app_version"), "{found:?}");
}

#[test]
fn a_tag_must_match() {
    let found = check(repo("1.2.3", "1.2.3", "1.0.0").path(), Some("v1.2.4")).unwrap();
    assert!(found[0].contains("tag v1.2.4"), "{found:?}");
}

#[test]
fn semver_needs_three_numeric_parts() {
    assert_eq!(parse_semver("1.20.3"), Some((1, 20, 3)));
    assert_eq!(parse_semver("1.2"), None);
    assert_eq!(parse_semver("1.2.3.4"), None);
    assert_eq!(parse_semver("1.2.x"), None);
}
