use super::*;
use crate::destination::Location;
use crate::manifest;

fn manifest_of(label: &str) -> (Family, PathBuf) {
    let target = targets(&Browser::ALL, Path::new("/Users/u"))
        .into_iter()
        .find(|t| t.label == label)
        .unwrap_or_else(|| panic!("no target labelled {label}"));
    let Location::File { manifest, .. } = target.location else {
        panic!("macOS registers files only");
    };
    (target.family, manifest)
}

#[test]
fn chrome_edge_and_firefox_folders_match_the_vendors_documentation() {
    let name = manifest::file_name();
    let base = PathBuf::from("/Users/u/Library/Application Support");
    assert_eq!(
        manifest_of("Google Chrome").1,
        base.join("Google/Chrome/NativeMessagingHosts").join(&name)
    );
    assert_eq!(
        manifest_of("Microsoft Edge Canary").1,
        base.join("Microsoft Edge Canary/NativeMessagingHosts")
            .join(&name)
    );
    let (family, firefox) = manifest_of("Firefox");
    assert_eq!(family, Family::Firefox);
    assert_eq!(
        firefox,
        base.join("Mozilla/NativeMessagingHosts").join(&name)
    );
}

#[test]
fn opera_also_writes_into_chromes_folder_once_its_own_exists() {
    let base = PathBuf::from("/Users/u/Library/Application Support");
    let gx = targets(&[Browser::Opera], Path::new("/Users/u"))
        .into_iter()
        .find(|t| t.label == "Opera GX (Chrome's folder)")
        .expect("Opera GX writes Chrome's folder");
    let Location::File {
        manifest, requires, ..
    } = gx.location
    else {
        panic!("files only");
    };
    assert_eq!(
        manifest,
        base.join("Google/Chrome/NativeMessagingHosts")
            .join(manifest::file_name())
    );
    assert_eq!(requires, Some(base.join("com.operasoftware.OperaGX")));
}

#[test]
fn every_target_requires_the_browsers_own_folder() {
    for target in targets(&Browser::ALL, Path::new("/Users/u")) {
        let Location::File {
            manifest, requires, ..
        } = target.location
        else {
            panic!("files only");
        };
        if target.label.ends_with("(Chrome's folder)") {
            continue;
        }
        assert!(manifest.starts_with(requires.expect("must require its root")));
    }
}

#[test]
fn brave_registers_only_in_chromes_folder_once_its_own_exists() {
    let base = PathBuf::from("/Users/u/Library/Application Support");
    let brave = targets(&[Browser::Brave], Path::new("/Users/u"));
    assert_eq!(brave.len(), 3, "one per channel");
    for target in brave {
        assert!(
            target.label.ends_with("(Chrome's folder)"),
            "{}",
            target.label
        );
        let Location::File {
            manifest, requires, ..
        } = target.location
        else {
            panic!("files only");
        };
        assert_eq!(
            manifest,
            base.join("Google/Chrome/NativeMessagingHosts")
                .join(manifest::file_name())
        );
        let root = requires.expect("must require Brave's folder");
        assert!(root.starts_with(base.join("BraveSoftware")), "{root:?}");
    }
}

#[test]
fn the_own_root_is_the_stable_channels_folder() {
    let home = Path::new("/Users/u");
    let base = PathBuf::from("/Users/u/Library/Application Support");
    assert_eq!(
        own_root(Browser::Brave, home),
        base.join("BraveSoftware/Brave-Browser")
    );
    assert_eq!(own_root(Browser::Firefox, home), base.join("Mozilla"));
    assert_eq!(
        own_root(Browser::Opera, home),
        base.join("com.operasoftware.Opera")
    );
}
