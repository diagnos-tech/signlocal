//! `cargo xtask check [summaries|i18n|generated|release]`.
//!
//! * `summaries` — every folder has a `SUMMARY.md` listing every entry
//!   exactly once (`docs/architecture/repository-layout.md` §SUMMARY.md);
//! * `i18n` — every locale has exactly the reference keys and placeholders
//!   (`websign_i18n::check`);
//! * `generated` — `cargo xtask gen` would change nothing;
//! * `release` — versions agree, notices exist, and the release binary does
//!   not contain the e2e marker.
//!
//! Every check reports all its problems at once: a CI log that stops at the
//! first one costs a round trip per problem.

mod generated;
mod glob;
mod i18n;
mod listing;
mod release;
mod summaries;
mod summary_file;
mod versions;

use crate::root::repo_root;

/// Checks in the order they run.
const ALL: [&str; 4] = ["summaries", "i18n", "generated", "release"];

/// `cargo xtask check`.
#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    /// Checks to run: `summaries`, `i18n`, `generated`, `release`; default all.
    pub checks: Vec<String>,
    /// `release`: the binary to inspect. Without it the binary check is
    /// skipped and only the release inputs are validated.
    #[arg(long)]
    pub binary: Option<std::path::PathBuf>,
}

/// Runs the checks; the error lists every violation.
pub fn run(args: &CheckArgs) -> Result<(), String> {
    if let Some(unknown) = args
        .checks
        .iter()
        .find(|name| !ALL.contains(&name.as_str()))
    {
        return Err(format!(
            "unknown check `{unknown}`; choose from: {}",
            ALL.join(", ")
        ));
    }
    let root = repo_root()?;
    let selected = |name: &str| args.checks.is_empty() || args.checks.iter().any(|c| c == name);

    let mut failures = Vec::new();
    for name in ALL.into_iter().filter(|name| selected(name)) {
        let problems = match name {
            "summaries" => summaries::check(&root),
            "i18n" => i18n::check(&root),
            "generated" => generated::check(&root),
            _ => release::check(&root, args.binary.as_deref()),
        }?;
        if problems.is_empty() {
            println!("check {name}: ok");
        } else {
            failures.push(format!("check {name}: {} problem(s)", problems.len()));
            failures.extend(problems.iter().map(|problem| format!("  {problem}")));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
