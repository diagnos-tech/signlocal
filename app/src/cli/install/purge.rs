//! `uninstall --purge`: settings, remembered sites and programs, usage and
//! logs. Only this app's own folders, found the same way the app finds them.
//!
//! A recursive delete is the one irreversible thing the installer does, so
//! a folder is deleted only when its path proves it is ours: absolute, at
//! least three components deep, and named after the app (`…/websign`) or
//! its log folder (`…/websign/logs`). An odd environment (`HOME=/`,
//! `XDG_CONFIG_HOME=` pointing at a shared folder) then fails the step
//! instead of deleting someone's files.

use std::path::{Component, Path};

use websign_registration::Outcome;

use crate::cli::report::{Kind, Step};

/// Deletes the data folder and the log folder.
pub fn purge(dry_run: bool) -> Vec<Step> {
    [
        (
            "settings and remembered sites",
            websign_host::store::data_dir(),
        ),
        ("logs", crate::logging::log_dir()),
    ]
    .into_iter()
    .filter_map(|(what, dir)| dir.map(|dir| (what, dir)))
    .map(|(what, dir)| Step {
        kind: Kind::Data,
        target: what.to_owned(),
        location: dir.display().to_string(),
        outcome: remove_dir(&dir, dry_run),
    })
    .collect()
}

fn remove_dir(dir: &Path, dry_run: bool) -> Outcome {
    if !is_our_folder(dir) {
        return Outcome::Failed("not this app's folder; left untouched".to_owned());
    }
    if !dir.exists() {
        return Outcome::NotPresent;
    }
    if dry_run {
        return Outcome::DryRun;
    }
    match std::fs::remove_dir_all(dir) {
        Ok(()) => Outcome::Removed,
        Err(error) => Outcome::Failed(format!("{:?}", error.kind())),
    }
}

/// Whether `dir` is shaped like one of the app's own folders.
fn is_our_folder(dir: &Path) -> bool {
    let names: Vec<&std::ffi::OsStr> = dir
        .components()
        .filter_map(|part| match part {
            Component::Normal(name) => Some(name),
            _ => None,
        })
        .collect();
    let plain = dir
        .components()
        .all(|part| !matches!(part, Component::ParentDir | Component::CurDir));
    let slug = std::ffi::OsStr::new(websign_project::SLUG);
    let ours = match names.as_slice() {
        [.., app, logs] if *logs == "logs" => *app == slug,
        [.., last] => *last == slug,
        [] => false,
    };
    dir.is_absolute() && plain && names.len() >= 3 && ours
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_folders_named_after_the_app_are_deleted() {
        for ours in [
            "/home/ana/.config/websign",
            "/home/ana/.local/state/websign",
            "/Users/ana/Library/Logs/websign",
        ] {
            assert!(is_our_folder(Path::new(ours)), "{ours}");
        }
        for other in [
            "",
            "/",
            "/websign",
            "/home/websign",
            "relative/dir/websign",
            "/home/ana/.config",
            "/home/ana/../websign/websign",
            "/home/ana/.config/websign/other",
            "/home/ana/.config/logs",
        ] {
            assert!(!is_our_folder(Path::new(other)), "{other}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_folders_are_recognized() {
        assert!(is_our_folder(Path::new(
            r"C:\Users\ana\AppData\Roaming\websign"
        )));
        assert!(is_our_folder(Path::new(
            r"C:\Users\ana\AppData\Local\websign\logs"
        )));
        assert!(!is_our_folder(Path::new(r"C:\websign")));
    }

    #[test]
    fn a_folder_that_is_not_ours_fails_the_step_and_stays() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(remove_dir(dir.path(), false), Outcome::Failed(_)));
        assert!(dir.path().exists());
    }
}
