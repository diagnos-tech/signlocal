//! `cargo xtask package --target <triple>`: release artifacts for one target
//! (`docs/architecture/packaging-and-release.md` §Artifacts).

/// `cargo xtask package`.
#[derive(Debug, clap::Args)]
pub struct PackageArgs {
    /// Rust target triple.
    #[arg(long)]
    pub target: String,
    /// Output folder (default `dist/`).
    #[arg(long)]
    pub out: Option<std::path::PathBuf>,
}

/// Builds and packs.
///
/// The command and its arguments exist so CI wiring is stable; the body
/// (nfpm, lipo, zips, installers) is implemented by the packaging track.
pub fn run(args: &PackageArgs) -> Result<(), String> {
    Err(format!(
        "`package --target {}` is implemented by the packaging track and is not available yet \
         (docs/architecture/packaging-and-release.md)",
        args.target
    ))
}
