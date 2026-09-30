//! `websign connect`: the framed protocol on stdin/stdout, identical to
//! native messaging except that the caller is the parent process.

use std::process::ExitCode;

/// `websign connect`.
#[derive(Debug, clap::Args)]
pub struct ConnectArgs {}

/// Identifies the parent process ([`crate::platform::caller`]) and serves
/// the connection until stdin closes or it idles out.
pub fn run(args: &ConnectArgs) -> ExitCode {
    let ConnectArgs {} = args;
    todo!("desktop-api.md §connect")
}
