use super::*;
use crate::destination::Location;
use crate::manifest;

const LIBS: [&str; 2] = ["/usr/lib", "/usr/lib64"];

fn libs() -> Vec<&'static Path> {
    LIBS.iter().map(Path::new).collect()
}

fn files(browsers: &[Browser]) -> Vec<(String, PathBuf, Option<PathBuf>)> {
    targets(browsers, &libs())
        .into_iter()
        .map(|target| match target.location {
            Location::File {
                manifest, requires, ..
            } => (target.label, manifest, requires),
            Location::Registry { .. } => panic!("Linux registers files only"),
        })
        .collect()
}

#[test]
fn each_browser_gets_the_documented_system_folder() {
    let name = manifest::file_name();
    let expected = [
        (Browser::Chrome, "/etc/opt/chrome/native-messaging-hosts"),
        (Browser::Chromium, "/etc/chromium/native-messaging-hosts"),
        (Browser::Edge, "/etc/opt/edge/native-messaging-hosts"),
        (Browser::Brave, "/etc/opt/chrome/native-messaging-hosts"),
        (Browser::Vivaldi, "/etc/opt/chrome/native-messaging-hosts"),
        (Browser::Opera, "/etc/opt/chrome/native-messaging-hosts"),
    ];
    for (browser, dir) in expected {
        let all = files(&[browser]);
        assert_eq!(all.len(), 1, "{browser:?}");
        assert_eq!(all[0].1, Path::new(dir).join(&name), "{browser:?}");
    }
}

#[test]
fn firefox_gets_one_folder_per_library_root() {
    let paths: Vec<PathBuf> = files(&[Browser::Firefox])
        .into_iter()
        .map(|f| f.1)
        .collect();
    let name = manifest::file_name();
    assert_eq!(
        paths,
        [
            Path::new("/usr/lib/mozilla/native-messaging-hosts").join(&name),
            Path::new("/usr/lib64/mozilla/native-messaging-hosts").join(&name),
        ]
    );
}

#[test]
fn shared_folders_are_written_once_and_name_every_browser() {
    let all = files(&Browser::ALL);
    let chrome: Vec<_> = all
        .iter()
        .filter(|(_, path, _)| path.starts_with("/etc/opt/chrome"))
        .collect();
    assert_eq!(chrome.len(), 1);
    assert_eq!(chrome[0].0, "Google Chrome, Brave, Vivaldi, Opera");
    let mut paths: Vec<_> = all.iter().map(|(_, path, _)| path.clone()).collect();
    paths.sort();
    paths.dedup();
    assert_eq!(paths.len(), all.len());
}

#[test]
fn system_targets_are_unconditional_and_carry_the_right_family() {
    for target in targets(&Browser::ALL, &libs()) {
        let Location::File {
            manifest,
            requires,
            host_copy,
        } = &target.location
        else {
            panic!("files only");
        };
        assert!(requires.is_none() && host_copy.is_none(), "{target:?}");
        let firefox = manifest.starts_with("/usr");
        assert_eq!(target.family == Family::Firefox, firefox, "{target:?}");
    }
}

#[test]
fn edge_reads_its_own_folder_then_chromes() {
    let name = manifest::file_name();
    let read: Vec<PathBuf> = manifests(Browser::Edge, &libs())
        .into_iter()
        .map(|(_, path)| path)
        .collect();
    assert_eq!(
        read,
        [
            Path::new("/etc/opt/edge/native-messaging-hosts").join(&name),
            Path::new("/etc/opt/chrome/native-messaging-hosts").join(&name),
        ]
    );
    assert_eq!(
        files(&[Browser::Edge]).len(),
        1,
        "only Edge's own folder is written"
    );
}
