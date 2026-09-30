//! `websign sign`: one signature from the command line.
//!
//! For digests that do not depend on the certificate (raw hash signing,
//! XAdES/JAdES built after choosing with `websign choose`). Formats that put
//! the certificate inside the signed data (PAdES, CAdES with
//! signing-certificate-v2) use `websign connect` or a client library, which
//! implement `prepare(certificate)`.

use std::path::PathBuf;
use std::process::ExitCode;

use super::options::{AlgorithmArg, HashArg};

/// `websign sign`.
#[derive(Debug, clap::Args)]
pub struct SignArgs {
    /// The hash the digest was computed with.
    #[arg(long, value_enum)]
    pub hash: HashArg,
    /// The digest, hex (any case, `:` allowed) or standard Base64.
    #[arg(
        long,
        conflicts_with = "digest_file",
        required_unless_present = "digest_file"
    )]
    pub digest: Option<String>,
    /// Read the raw digest bytes from a file, or `-` for stdin.
    #[arg(long, value_name = "PATH")]
    pub digest_file: Option<PathBuf>,
    /// Acceptable algorithms, preferred first (repeatable).
    #[arg(long = "algorithm", value_enum, value_name = "ALG")]
    pub algorithms: Vec<AlgorithmArg>,
    /// Preselect this certificate (SHA-256 fingerprint, hex).
    #[arg(long, value_name = "FINGERPRINT")]
    pub certificate: Option<String>,
}

/// Prints the final message (`sign.result` or `error`) as one JSON object
/// and exits with its code.
pub fn run(args: &SignArgs) -> ExitCode {
    let _ = args;
    todo!("desktop-api.md §sign")
}
