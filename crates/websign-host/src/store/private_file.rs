//! Files only their owner can read. The stores hold which sites may receive
//! the person's certificates without asking, so on Unix they are created
//! `0600` in a `0700` folder whatever the umask says; on Windows the
//! per-user profile folder's ACL already keeps other users out.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

/// Creates `dir` and its missing parents, owner-only where they are new.
pub(super) fn create_dir(dir: &Path) -> io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)
}

/// Opens `path` for writing, creating it owner-only; `truncate` empties it.
pub(super) fn open(path: &Path, truncate: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(truncate);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
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
}
