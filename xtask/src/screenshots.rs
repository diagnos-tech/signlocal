//! `cargo xtask screenshots --from <dir> --os <name>`: copies e2e PNGs into
//! `docs/screenshots/<os>/` with stable names and rewrites that folder's
//! `SUMMARY.md` and index page (`docs/architecture/testing.md` §Screenshots).

/// `cargo xtask screenshots`.
#[derive(Debug, clap::Args)]
pub struct ScreenshotsArgs {
    #[arg(long)]
    pub from: std::path::PathBuf,
    /// `windows`, `macos`, `ubuntu-24.04`, `fedora-42`, …
    #[arg(long)]
    pub os: String,
}

/// Copies and indexes.
pub fn run(args: &ScreenshotsArgs) -> Result<(), String> {
    let _ = args;
    todo!("testing.md §Screenshots")
}
