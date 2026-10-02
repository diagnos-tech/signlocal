//! The log file, size-capped by rotation.
//!
//! Several processes may log at once (two browsers, a desktop client), so
//! the file is opened for appending and each line is one `write` call.
//! After another process rotated, this one's handle points at the renamed
//! `websign.log.1`: every write first compares the handle's size with the
//! file under the name, reopens the name when they differ, and decides on
//! rotation from the file under the name. Otherwise the second process
//! would keep growing the old file and then rotate again, deleting the
//! lines the first one just moved aside.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Size at which `websign.log` becomes `websign.log.1`, replacing the older
/// one: at most twice this on disk.
pub const MAX_BYTES: u64 = 1024 * 1024;

const FILE_NAME: &str = "websign.log";

/// An open log file.
#[derive(Debug)]
pub struct Sink {
    path: PathBuf,
    file: File,
}

impl Sink {
    /// Opens (creating) the log in `dir`; `None` when the folder is not
    /// writable.
    pub fn open(dir: &Path) -> Option<Sink> {
        create_private_dir(dir).ok()?;
        let path = dir.join(FILE_NAME);
        let file = open_append(&path).ok()?;
        Some(Sink { path, file })
    }

    /// Appends `line`, rotating first when it would pass [`MAX_BYTES`].
    /// Failures are ignored: logging must never fail the app.
    pub fn write_line(&mut self, line: &str) {
        let named = std::fs::metadata(&self.path).map_or(0, |m| m.len());
        let ours = self.file.metadata().map_or(0, |m| m.len());
        if ours != named {
            self.reopen();
        }
        if named + line.len() as u64 > MAX_BYTES {
            let _ = std::fs::rename(&self.path, self.path.with_extension("log.1"));
            self.reopen();
        }
        let _ = self.file.write_all(line.as_bytes());
    }

    fn reopen(&mut self) {
        if let Ok(file) = open_append(&self.path) {
            self.file = file;
        }
    }
}

/// The folder, readable by this user only (Unix). A folder that already
/// exists (made by an older version, or by hand) is tightened too.
fn create_private_dir(dir: &Path) -> std::io::Result<()> {
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

/// Appending, readable by this user only (Unix).
fn open_append(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_keeps_at_most_two_files_under_the_cap() {
        let dir = tempfile::tempdir().unwrap();
        let mut sink = Sink::open(dir.path()).unwrap();
        let line = "x".repeat(1000) + "\n";
        for _ in 0..(3 * MAX_BYTES / 1000) {
            sink.write_line(&line);
        }
        let current = std::fs::metadata(dir.path().join(FILE_NAME)).unwrap().len();
        let old = std::fs::metadata(dir.path().join("websign.log.1"))
            .unwrap()
            .len();
        assert!(current <= MAX_BYTES && old <= MAX_BYTES, "{current} {old}");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    }

    #[test]
    fn a_second_writer_follows_the_rotation_instead_of_repeating_it() {
        let dir = tempfile::tempdir().unwrap();
        let mut first = Sink::open(dir.path()).unwrap();
        let mut second = Sink::open(dir.path()).unwrap();
        let line = "x".repeat(999) + "\n";
        let lines = MAX_BYTES / 1000;
        for _ in 0..lines {
            first.write_line(&line);
        }
        // Passes the cap: the first writer rotates.
        first.write_line(&line);
        second.write_line("written by the second\n");
        let current = std::fs::read_to_string(dir.path().join(FILE_NAME)).unwrap();
        let old = std::fs::read_to_string(dir.path().join("websign.log.1")).unwrap();
        assert_eq!(old.len() as u64, lines * 1000);
        assert_eq!(current, line + "written by the second\n");
    }

    #[cfg(unix)]
    #[test]
    fn the_file_is_private_to_the_user() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let logs = dir.path().join("logs");
        Sink::open(&logs).unwrap().write_line("step\n");
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&logs), 0o700);
        assert_eq!(mode(&logs.join(FILE_NAME)), 0o600);
        std::fs::set_permissions(&logs, std::fs::Permissions::from_mode(0o755)).unwrap();
        Sink::open(&logs).unwrap();
        assert_eq!(mode(&logs), 0o700);
    }
}
