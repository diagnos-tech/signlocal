//! `websign choose`: the certificate the person picks, as JSON.

use std::process::ExitCode;

use super::options::AlgorithmArg;

/// `websign choose`.
#[derive(Debug, clap::Args)]
pub struct ChooseArgs {
    /// Only certificates whose key can produce one of these (repeatable).
    #[arg(long = "algorithm", value_enum, value_name = "ALG")]
    pub algorithms: Vec<AlgorithmArg>,
}

/// Prints `choose.result` or `error` as one JSON object.
pub fn run(args: &ChooseArgs) -> ExitCode {
    let _ = args;
    todo!("desktop-api.md §choose")
}
