//! `websign sign`: one signature from the command line.
//!
//! For digests that do not depend on the certificate (raw hash signing,
//! XAdES/JAdES built after choosing with `websign choose`). Formats that put
//! the certificate inside the signed data (PAdES, CAdES with
//! signing-certificate-v2) use `websign connect` or a client library, which
//! implement `prepare(certificate)`.

mod digest;

use std::path::PathBuf;
use std::process::ExitCode;

use websign_protocol::messages::{ClientMessage, SignBegin};
use websign_protocol::types::{FingerprintHex, HashName};
use websign_protocol::{AppMessage, ErrorCode};

use super::options::{AlgorithmArg, HashArg};
use super::output::{error, finish};

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
    let message = match prepare(args) {
        Ok((begin, digest)) => super::local::request(ClientMessage::SignBegin(begin), Some(digest)),
        Err(refusal) => *refusal,
    };
    finish(&message, &crate::ui::i18n::catalog())
}

/// The request and its digest, or the `InvalidRequest` error to print
/// before any window opens (a malformed digest is the caller's bug, not a
/// question for the person).
pub fn prepare(args: &SignArgs) -> Result<(SignBegin, Vec<u8>), Box<AppMessage>> {
    let hash = HashName::from(args.hash);
    let invalid = |message: String| Box::new(error(ErrorCode::InvalidRequest, message));
    let digest = match (&args.digest, &args.digest_file) {
        (Some(text), _) => digest::parse_text(text, hash),
        (None, Some(path)) => digest::read_file(path, hash),
        (None, None) => Err("a digest is required".to_owned()),
    }
    .map_err(invalid)?;
    let certificate = args
        .certificate
        .as_deref()
        .map(|fp| FingerprintHex::new(fp.replace(':', "").to_ascii_lowercase()))
        .transpose()
        .map_err(|_| invalid("--certificate is not a SHA-256 fingerprint in hex".to_owned()))?;
    let algorithms =
        (!args.algorithms.is_empty()).then(|| args.algorithms.iter().map(|&a| a.into()).collect());
    let begin = SignBegin {
        web: None,
        hash,
        algorithms,
        certificate,
    };
    Ok((begin, digest))
}
