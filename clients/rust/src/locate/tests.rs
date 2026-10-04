use std::fs;
use std::path::PathBuf;

use super::*;

/// A scratch directory removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Scratch {
        let dir = env::temp_dir().join(format!("websign-client-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    fn file(&self, relative: &str) -> PathBuf {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"").unwrap();
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn override_wins_over_path() {
    let dir = Scratch::new("override");
    let chosen = dir.file("chosen/app");
    let on_path = dir.file("bin/websign");
    let inputs = SearchInputs {
        override_path: Some(chosen.clone().into()),
        path: Some(on_path.parent().unwrap().into()),
        ..SearchInputs::default()
    };
    assert_eq!(search(&inputs, Platform::Linux), Some(chosen));
}

#[test]
fn missing_override_falls_back_to_path() {
    let dir = Scratch::new("fallback");
    let on_path = dir.file("bin/websign");
    let inputs = SearchInputs {
        override_path: Some(dir.0.join("nowhere").into()),
        path: Some(on_path.parent().unwrap().into()),
        ..SearchInputs::default()
    };
    assert_eq!(search(&inputs, Platform::Linux), Some(on_path));
}

#[test]
fn relative_path_entries_are_ignored() {
    let path = env::join_paths(["", ".", "bin"]).unwrap();
    assert_eq!(on_path(Some(&path), Platform::Linux), None);
}

#[test]
fn falls_back_to_the_install_locations() {
    let home = Scratch::new("home");
    let installed = home.file(".local/bin/websign");
    let inputs = SearchInputs {
        home: Some(home.0.clone()),
        ..SearchInputs::default()
    };
    // /usr/bin/websign may exist on a machine with the app installed.
    let found = search(&inputs, Platform::Linux).unwrap();
    assert!(found == installed || found == Path::new("/usr/bin/websign"));
}

#[test]
fn nothing_found_is_none() {
    assert_eq!(search(&SearchInputs::default(), Platform::MacOs), None);
}

#[test]
fn locations_match_the_documentation() {
    let inputs = SearchInputs {
        home: Some(PathBuf::from("/h")),
        local_app_data: Some(PathBuf::from("/l")),
        ..SearchInputs::default()
    };
    assert_eq!(
        install_locations(&inputs, Platform::Linux),
        [
            PathBuf::from("/usr/bin/websign"),
            PathBuf::from("/h/.local/bin/websign")
        ]
    );
    assert_eq!(
        install_locations(&inputs, Platform::MacOs),
        [
            PathBuf::from("/Applications/SignLocal.app/Contents/MacOS/websign"),
            PathBuf::from("/h/Applications/SignLocal.app/Contents/MacOS/websign"),
            PathBuf::from("/h/.local/bin/websign"),
        ]
    );
    let base = PathBuf::from("/l");
    assert_eq!(
        install_locations(&inputs, Platform::Windows),
        [
            base.join("Programs").join("SignLocal").join("websign.exe"),
            base.join("Microsoft")
                .join("WindowsApps")
                .join("websign.exe"),
        ]
    );
}
