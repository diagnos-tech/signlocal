//! Finding the `websign` executable.

use std::path::PathBuf;

/// The app executable: `WEBSIGN_EXECUTABLE` when set, else `websign` on
/// `PATH`, else the install locations of `docs/architecture/
/// packaging-and-release.md` §Install locations for this OS.
pub fn find_executable() -> Option<PathBuf> {
    todo!("SPEC.md §1")
}
