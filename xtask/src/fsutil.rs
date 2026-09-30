//! File-system helpers whose errors say which path and what was attempted,
//! because a bare `os error 2` is useless in a CI log.

use std::fs;
use std::path::Path;

/// Reads a UTF-8 file.
pub fn read_text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

/// Whether the file text `current` equals `wanted` up to line endings: a
/// Windows checkout with `core.autocrlf` turns `\n` into `\r\n`, and that
/// must not count as a change.
pub fn same_text(current: &str, wanted: &str) -> bool {
    current == wanted || current.replace("\r\n", "\n") == wanted
}

/// Writes `content`, creating parent folders. Skips the write when the file
/// already has this text so timestamps (and watchers) stay quiet.
pub fn write_text(path: &Path, content: &str) -> Result<(), String> {
    if fs::read_to_string(path).is_ok_and(|current| same_text(&current, content)) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    fs::write(path, content).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// Names of the entries of `folder`, sorted, with a flag for folders.
pub fn list_dir(folder: &Path) -> Result<Vec<(String, bool)>, String> {
    let read =
        fs::read_dir(folder).map_err(|e| format!("cannot list {}: {e}", folder.display()))?;
    let mut entries = Vec::new();
    for entry in read {
        let entry = entry.map_err(|e| format!("cannot list {}: {e}", folder.display()))?;
        let is_dir = entry.path().is_dir();
        entries.push((entry.file_name().to_string_lossy().into_owned(), is_dir));
    }
    entries.sort();
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_checkouts_count_as_the_same_text() {
        assert!(same_text("a\r\nb\r\n", "a\nb\n"));
        assert!(!same_text("a\nc\n", "a\nb\n"));
    }
}
