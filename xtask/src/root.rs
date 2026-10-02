//! Locating the repository root, so every command works from any folder.

use std::path::{Path, PathBuf};

/// The repository root: the folder that holds `project.toml`.
///
/// `cargo xtask` sets `CARGO_MANIFEST_DIR` at build time, which is right even
/// when the binary is started from a subfolder; walking up from the current
/// folder covers a binary that was moved.
pub fn repo_root() -> Result<PathBuf, String> {
    let built_in = Path::new(env!("CARGO_MANIFEST_DIR")).parent();
    if let Some(root) = built_in.filter(|root| is_root(root)) {
        return Ok(root.to_path_buf());
    }
    let start =
        std::env::current_dir().map_err(|e| format!("cannot read the current folder: {e}"))?;
    start
        .ancestors()
        .find(|folder| is_root(folder))
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            "cannot find the repository root (no project.toml above the current folder); \
             run `cargo xtask` from inside the repository"
                .to_owned()
        })
}

fn is_root(folder: &Path) -> bool {
    folder.join("project.toml").is_file() && folder.join("Cargo.toml").is_file()
}
