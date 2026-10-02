//! `websign choose`: the certificate the person picks, as JSON.

use std::process::ExitCode;

use websign_protocol::messages::{Choose, ClientMessage};
use websign_protocol::types::CertificateFilter;

use super::options::AlgorithmArg;
use super::output::finish;

/// `websign choose`.
#[derive(Debug, clap::Args)]
pub struct ChooseArgs {
    /// Only certificates whose key can produce one of these (repeatable).
    #[arg(long = "algorithm", value_enum, value_name = "ALG")]
    pub algorithms: Vec<AlgorithmArg>,
}

/// Prints `choose.result` or `error` as one JSON object.
pub fn run(args: &ChooseArgs) -> ExitCode {
    let message = super::local::request(ClientMessage::Choose(request(args)), None);
    finish(&message, &crate::ui::i18n::catalog())
}

/// The `choose` request; no filter when no algorithm is named (the protocol
/// forbids an empty list).
pub fn request(args: &ChooseArgs) -> Choose {
    let filter = (!args.algorithms.is_empty()).then(|| CertificateFilter {
        algorithms: Some(args.algorithms.iter().map(|&a| a.into()).collect()),
    });
    Choose { web: None, filter }
}
