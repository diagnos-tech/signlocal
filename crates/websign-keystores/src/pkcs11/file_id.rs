//! Identity of a module file, so the same library reached through symlinks,
//! hard links or a different spelling of its path is loaded only once.

use std::path::Path;

/// Two paths with the same `FileId` are the same file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileId(String);

impl FileId {
    /// `None` when the file does not exist or cannot be inspected.
    pub fn of(path: &Path) -> Option<FileId> {
        let canonical = std::fs::canonicalize(path).ok()?;
        Some(FileId(identity(&canonical)?))
    }
}

/// Device and inode catch hard links, which canonical paths cannot.
#[cfg(unix)]
fn identity(canonical: &Path) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(canonical).ok()?;
    Some(format!("{}:{}", metadata.dev(), metadata.ino()))
}

/// Windows paths are case-insensitive; canonicalizing already resolved links.
#[cfg(not(unix))]
fn identity(canonical: &Path) -> Option<String> {
    Some(canonical.to_string_lossy().to_lowercase())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("websign-fileid-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn symlinks_and_hard_links_share_an_identity() {
        let dir = scratch_dir("links");
        let original = dir.join("libmodule.so");
        std::fs::write(&original, b"x").unwrap();
        let symlink = dir.join("libmodule-link.so");
        std::os::unix::fs::symlink(&original, &symlink).unwrap();
        let hard = dir.join("libmodule-hard.so");
        std::fs::hard_link(&original, &hard).unwrap();
        let other = dir.join("libother.so");
        std::fs::write(&other, b"x").unwrap();

        let id = FileId::of(&original).unwrap();
        assert_eq!(FileId::of(&symlink), Some(id.clone()));
        assert_eq!(FileId::of(&hard), Some(id.clone()));
        assert_ne!(FileId::of(&other), Some(id));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_file_has_no_identity() {
        assert_eq!(FileId::of(Path::new("/nonexistent/libnothing.so")), None);
    }
}
