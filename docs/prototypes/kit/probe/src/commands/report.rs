//! `websign-probe report`: one Markdown file with everything a proof needs,
//! and nothing personal.

use std::process::ExitCode;

/// Arguments of `websign-probe report`.
#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let _ = args;
    todo!("report command")
}
