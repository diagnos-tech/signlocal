//! Files only their owner can read. The stores hold which sites may receive
//! the person's certificates without asking, so on Unix they are `0600` in
//! a `0700` folder whatever the umask says; on Windows the per-user profile
//! folder's ACL already keeps other users out.
//!
//! Modes are set on every write, not only on creation (like the log folder
//! in `app/src/logging/sink.rs`): a folder made by an older version or by
//! hand, or a `*.tmp` left by a crash under another umask, is tightened the
//! next time a store is written.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

/// Creates `dir` and its missing parents, and makes `dir` itself
/// owner-only, whether it is new or not.
pub(super) fn create_dir(dir: &Path) -> io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Opens `path` for writing, owner-only whether it is new or left over;
/// `truncate` empties it.
pub(super) fn open(path: &Path, truncate: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(truncate);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let file = options.open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(file)
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    fn chmod(path: &Path, mode: u32) {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    #[test]
    fn files_and_folders_are_owner_only() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("a").join("b");
        create_dir(&dir).unwrap();
        assert_eq!(mode(&dir), 0o700);
        let file = dir.join("f.json");
        open(&file, true).unwrap();
        assert_eq!(mode(&file), 0o600);
    }

    #[test]
    fn an_existing_folder_and_a_leftover_file_are_tightened() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("store");
        std::fs::create_dir(&dir).unwrap();
        chmod(&dir, 0o755);
        let leftover = dir.join("consent.json.tmp");
        std::fs::write(&leftover, b"{}").unwrap();
        chmod(&leftover, 0o644);
        create_dir(&dir).unwrap();
        assert_eq!(mode(&dir), 0o700);
        open(&leftover, true).unwrap();
        assert_eq!(mode(&leftover), 0o600);
    }
}
