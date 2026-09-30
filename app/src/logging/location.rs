//! Where the log lives: the OS's per-user log folder.
//!
//! Per user, never the shared temp folder: a predictable file name in
//! `/tmp` could be pre-created or linked by another account, and the log of
//! one person must not be readable by another.

use std::path::PathBuf;

/// `%LOCALAPPDATA%\websign\logs` (Windows), `~/Library/Logs/websign`
/// (macOS), `$XDG_STATE_HOME/websign` or `~/.local/state/websign` (Linux).
/// `None` when the environment names no such folder.
pub fn log_dir() -> Option<PathBuf> {
    let slug = websign_project::SLUG;
    let var = |name: &str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    if cfg!(windows) {
        var("LOCALAPPDATA").map(|dir| dir.join(slug).join("logs"))
    } else if cfg!(target_os = "macos") {
        var("HOME").map(|home| home.join("Library/Logs").join(slug))
    } else {
        var("XDG_STATE_HOME")
            .or_else(|| var("HOME").map(|home| home.join(".local/state")))
            .map(|dir| dir.join(slug))
    }
}
