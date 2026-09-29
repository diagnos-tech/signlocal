//! `websign-probe host`: the native messaging loop, started by hand.

use std::process::ExitCode;

/// Arguments of `websign-probe host`.
#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let _ = args;
    todo!("manual host")
}
