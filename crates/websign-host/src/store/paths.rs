//! Where the files live.

use std::path::PathBuf;

/// `%APPDATA%\websign` (Windows), `~/Library/Application Support/websign`
/// (macOS; inside the container when sandboxed, which is fine: only the app
/// reads it), `$XDG_CONFIG_HOME/websign` (Linux). `None` when the OS gives no
/// folder; the app then runs without persistence.
pub fn data_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(websign_project::SLUG))
}
