//! Short-lived private files handed to another program (the certificate
//! viewer reads a file, not our memory).
//!
//! They live in `$XDG_RUNTIME_DIR/<slug>/`: a per-user tmpfs the session
//! wipes at logout, in a folder only we can enter. Files older than
//! [`MAX_AGE`] are removed whenever a new one is written, which covers
//! viewers that outlive the app.

use std::fs::{DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const MAX_AGE: Duration = Duration::from_secs(10 * 60);

/// Writes `bytes` to a new file readable only by us, named
/// `<stem>-<unique>.<extension>`; `None` when no private folder is usable.
pub fn write(stem: &str, extension: &str, bytes: &[u8]) -> Option<PathBuf> {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .unwrap_or_else(std::env::temp_dir);
    write_in(&base, stem, extension, bytes)
}

fn write_in(base: &Path, stem: &str, extension: &str, bytes: &[u8]) -> Option<PathBuf> {
    let dir = private_dir(base)?;
    remove_old(&dir);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    let path = dir.join(format!(
        "{stem}-{}-{unique}.{extension}",
        std::process::id()
    ));
    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .and_then(|mut file| file.write_all(bytes));
    match written {
        Ok(()) => Some(path),
        Err(_) => {
            let _ = std::fs::remove_file(&path);
            None
        }
    }
}

/// `<base>/<slug>`, created `0700`; refused unless it is a real folder we
/// own that nobody else can enter (the shared `/tmp` fallback could hold a
/// folder or symlink another user planted).
fn private_dir(base: &Path) -> Option<PathBuf> {
    let dir = base.join(websign_project::SLUG);
    let _ = DirBuilder::new().mode(0o700).create(&dir);
    let meta = std::fs::symlink_metadata(&dir).ok()?;
    // SAFETY: `getuid` has no preconditions and cannot fail.
    let uid = unsafe { libc::getuid() };
    (meta.is_dir() && meta.uid() == uid && meta.mode() & 0o077 == 0).then_some(dir)
}

fn remove_old(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let old = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|modified| modified.elapsed().ok())
            .is_some_and(|age| age > MAX_AGE);
        if old {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn base(test: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("websign-private-{}-{test}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn writes_owner_only_files_in_an_owner_only_folder() {
        let base = base("write");
        let path = write_in(&base, "certificate", "crt", b"der").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"der");
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&path), 0o600);
        assert_eq!(mode(path.parent().unwrap()), 0o700);
        assert_eq!(path.extension().unwrap(), "crt");
        let second = write_in(&base, "certificate", "crt", b"der").unwrap();
        assert_ne!(path, second);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn refuses_a_folder_others_can_enter() {
        let base = base("shared");
        let dir = base.join(websign_project::SLUG);
        std::fs::create_dir(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(write_in(&base, "certificate", "crt", b"der"), None);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn removes_files_older_than_the_limit() {
        let base = base("sweep");
        let old = write_in(&base, "certificate", "crt", b"old").unwrap();
        let file = std::fs::File::options().write(true).open(&old).unwrap();
        file.set_modified(SystemTime::now() - MAX_AGE - Duration::from_secs(60))
            .unwrap();
        let fresh = write_in(&base, "certificate", "crt", b"new").unwrap();
        assert!(!old.exists());
        assert!(fresh.exists());
        std::fs::remove_dir_all(base).unwrap();
    }
}
