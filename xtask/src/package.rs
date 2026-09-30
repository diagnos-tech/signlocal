//! `cargo xtask package --target <triple>`: release artifacts for one target
//! (`docs/architecture/packaging-and-release.md` §Artifacts), written to
//! `dist/` with the exact names the installers and the release job expect.

mod archive;
mod build;
mod extension;
mod formats;
mod linux;
mod macos;
mod macos_sign;
mod names;
mod nfpm;
pub(crate) mod project;
mod safari;
mod sha256;
mod stage;
mod sums;
mod target;
mod template;
mod tool;
mod windows;

use std::path::PathBuf;

use crate::root::repo_root;
use formats::Format;

/// `cargo xtask package`.
#[derive(Debug, clap::Args)]
pub struct PackageArgs {
    /// Rust target triple (`universal-apple-darwin` for the macOS app).
    /// Optional only with `--sums-only`.
    #[arg(long, required_unless_present = "sums_only")]
    pub target: Option<String>,
    /// Output folder (default `dist/`).
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// What to produce (default: everything the target's OS ships).
    #[arg(long, value_enum)]
    pub format: Vec<Format>,
    /// Use the binaries already in the cargo target folder instead of building.
    #[arg(long)]
    pub no_build: bool,
    /// Afterwards write `SHA256SUMS` for every file in the output folder,
    /// adding the install scripts. Run it once, after all artifacts are there.
    #[arg(long)]
    pub sums: bool,
    /// Only write `SHA256SUMS` (the release job, after collecting artifacts).
    #[arg(long)]
    pub sums_only: bool,
}

/// Everything a packaging step needs, resolved once.
pub struct Context {
    pub root: PathBuf,
    pub out: PathBuf,
    pub project: project::Project,
    pub version: String,
    pub no_build: bool,
}

/// Builds and packs.
pub fn run(args: &PackageArgs) -> Result<(), String> {
    let root = repo_root()?;
    let out = args.out.clone().unwrap_or_else(|| root.join("dist"));
    std::fs::create_dir_all(&out).map_err(|e| format!("cannot create {}: {e}", out.display()))?;
    if let Some(triple) = &args.target {
        let (project, version) = project::load(&root)?;
        let context = Context {
            root,
            out: out.clone(),
            project,
            version,
            no_build: args.no_build,
        };
        let target = target::parse(triple)?;
        for format in formats::plan(&target, &args.format)? {
            let artifact = produce(&context, &target, format)?;
            println!("packaged {}", artifact.display());
        }
    }
    if args.sums || args.sums_only {
        let file = sums::write(&repo_root()?, &out)?;
        println!("wrote {}", file.display());
    }
    Ok(())
}

fn produce(context: &Context, target: &target::Target, format: Format) -> Result<PathBuf, String> {
    match format {
        Format::Deb | Format::Rpm => nfpm::package(context, target, format),
        Format::TarGz => linux::tarball(context, target),
        Format::Zip => windows::zip(context, target),
        Format::App => macos::app(context),
        Format::Extension => extension::zips(context),
    }
}
