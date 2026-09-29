//! Windows packaging facts.

use std::path::PathBuf;

/// When running from an MSIX package, the app execution alias browsers must
/// start instead of the real binary (files under `WindowsApps` cannot be
/// launched directly). `None` when not packaged.
pub fn msix_alias_path() -> Option<PathBuf> {
    todo!("msix alias path")
}
