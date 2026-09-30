//! `cargo xtask <command>`: every generation and check step behind one entry
//! point, so CI and people run the same thing (`docs/architecture/testing.md`
//! §CI gates).

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

mod check;
mod fsutil;
mod generate;
mod i18n_files;
mod package;
mod plan;
mod root;
mod screenshots;
#[cfg(test)]
mod testutil;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Repository automation.
#[derive(Debug, Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Regenerate every derived file (TypeScript types, extension locales,
    /// SDK messages, project constants for TypeScript).
    Gen(generate::GenArgs),
    /// Verify repository invariants (SUMMARY.md, i18n, generated files, e2e
    /// feature absent from release builds).
    Check(check::CheckArgs),
    /// Build release artifacts for one target.
    Package(package::PackageArgs),
    /// Collect e2e screenshots into docs/screenshots/<os>/.
    Screenshots(screenshots::ScreenshotsArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Gen(args) => generate::run(&args),
        Command::Check(args) => check::run(&args),
        Command::Package(args) => package::run(&args),
        Command::Screenshots(args) => screenshots::run(&args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}
