use super::*;
use crate::testutil::TempDir;

fn png_bytes(dir: &TempDir, relative: &str) {
    let path = dir.path().join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut bytes = PNG_SIGNATURE.to_vec();
    bytes.extend_from_slice(b"data");
    std::fs::write(path, bytes).unwrap();
}

#[test]
fn copies_pngs_and_writes_index_and_summaries() {
    let repo = TempDir::new();
    let from = TempDir::new();
    png_bytes(&from, "run1/confirm-waiting-light.png");
    png_bytes(&from, "confirm-waiting-dark.png");
    from.write("notes.txt", "ignored");

    assert_eq!(
        publish(repo.path(), from.path(), "ubuntu-24.04").unwrap(),
        2
    );

    let os = repo.path().join("docs/screenshots/ubuntu-24.04");
    assert!(os.join("confirm-waiting-light.png").is_file());
    let index = std::fs::read_to_string(os.join("index.md")).unwrap();
    assert!(
        index.contains("| waiting | ![confirm-waiting-light.png](confirm-waiting-light.png) |")
    );
    let summary = std::fs::read_to_string(os.join("SUMMARY.md")).unwrap();
    assert!(summary.contains("- `confirm-waiting-dark.png` — confirm window, waiting, dark theme"));
    let root = std::fs::read_to_string(repo.path().join("docs/screenshots/SUMMARY.md")).unwrap();
    assert!(root.contains("- `ubuntu-24.04/`"));
}

#[test]
fn a_rerun_replaces_images_that_no_longer_exist() {
    let repo = TempDir::new();
    let first = TempDir::new();
    png_bytes(&first, "a-x-light.png");
    publish(repo.path(), first.path(), "macos").unwrap();
    let second = TempDir::new();
    png_bytes(&second, "b-y-light.png");
    publish(repo.path(), second.path(), "macos").unwrap();
    assert!(
        !repo
            .path()
            .join("docs/screenshots/macos/a-x-light.png")
            .exists()
    );
}

#[test]
fn bad_input_is_refused_with_a_reason() {
    let repo = TempDir::new();
    let empty = TempDir::new();
    assert!(
        publish(repo.path(), empty.path(), "macos")
            .unwrap_err()
            .contains("no PNG files")
    );
    assert!(
        publish(repo.path(), empty.path(), "../x")
            .unwrap_err()
            .contains("--os")
    );

    let fake = TempDir::new();
    fake.write("a-x-light.png", "not a png");
    assert!(
        publish(repo.path(), fake.path(), "macos")
            .unwrap_err()
            .contains("not a PNG")
    );

    let twins = TempDir::new();
    png_bytes(&twins, "one/a-x-light.png");
    png_bytes(&twins, "two/a-x-light.png");
    assert!(
        publish(repo.path(), twins.path(), "macos")
            .unwrap_err()
            .contains("another folder")
    );
}

#[test]
fn the_root_summary_describes_popup_as_popup_states_not_an_os() {
    let repo = TempDir::new();
    std::fs::create_dir_all(repo.path().join("docs/screenshots/popup")).unwrap();
    let from = TempDir::new();
    png_bytes(&from, "a-x-light.png");
    publish(repo.path(), from.path(), "macos").unwrap();

    let root = std::fs::read_to_string(repo.path().join("docs/screenshots/SUMMARY.md")).unwrap();
    assert!(root.contains(
        "- `popup/` — every extension popup state, light and dark, and the missing state in 7 locales\n"
    ));
    assert!(root.contains("- `macos/` — every window state on macos, light and dark\n"));
}
