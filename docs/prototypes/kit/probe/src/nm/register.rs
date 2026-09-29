//! `websign-probe register`: writes the native messaging host manifest where
//! each installed browser looks for it.

use std::process::ExitCode;

/// Arguments of `websign-probe register`.
#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let _ = args;
    todo!("register command")
}
