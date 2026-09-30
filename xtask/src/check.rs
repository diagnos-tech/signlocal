//! `cargo xtask check [summaries|i18n|generated|release]`.
//!
//! * `summaries` — every folder has a `SUMMARY.md` listing every entry
//!   exactly once (`docs/architecture/repository-layout.md` §SUMMARY.md);
//! * `i18n` — every locale has exactly the reference keys and placeholders
//!   (`websign_i18n::check`);
//! * `generated` — `cargo xtask gen` would change nothing;
//! * `release` — the release binary does not contain the e2e marker.

/// `cargo xtask check`.
#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    /// Checks to run; default all.
    pub checks: Vec<String>,
    /// `release`: the binary to inspect.
    #[arg(long)]
    pub binary: Option<std::path::PathBuf>,
}

/// Runs the checks; the error lists every violation.
pub fn run(args: &CheckArgs) -> Result<(), String> {
    let _ = args;
    todo!("repository-layout.md §SUMMARY.md, i18n.md §CI")
}
