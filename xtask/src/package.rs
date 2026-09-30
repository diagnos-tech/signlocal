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
pub fn run(args: &PackageArgs) -> Result<(), String> {
    let _ = args;
    todo!("packaging-and-release.md §Artifacts")
}
