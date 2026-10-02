//! `websign connect`: the framed protocol on stdin/stdout, identical to
//! native messaging except that the caller is the parent process.

use std::process::ExitCode;

use websign_protocol::framing::{read_frame, write_frame};
use websign_protocol::{
    AppEnvelope, ErrorCode, WireError, parse_client_message, refusal_version, to_json,
};

/// `websign connect`.
#[derive(Debug, clap::Args)]
pub struct ConnectArgs {}

/// Identifies the parent process ([`crate::platform::caller`]) and serves
/// the connection until stdin closes or it idles out.
pub fn run(args: &ConnectArgs) -> ExitCode {
    let ConnectArgs {} = args;
    match crate::platform::caller::parent_caller() {
        Some(caller) => crate::host_process::serve_desktop(caller),
        None => refuse_first_frame(),
    }
}

/// Without a caller to show, nothing may be signed: the first request gets
/// `Internal` (at a version its sender can read) and the process exits 1.
fn refuse_first_frame() -> ExitCode {
    let mut stdin = std::io::stdin().lock();
    if let Ok(Some(frame)) = read_frame(&mut stdin) {
        let id = match parse_client_message(&frame, None) {
            Ok(envelope) => Some(envelope.id),
            Err(error) => error.id,
        };
        if let Some(id) = id {
            let reply = AppEnvelope {
                v: refusal_version(&frame),
                id,
                message: websign_protocol::AppMessage::Error(WireError {
                    code: ErrorCode::Internal,
                    message: "the calling program could not be identified".to_owned(),
                    details: None,
                }),
            };
            let _ = write_frame(&mut std::io::stdout().lock(), &to_json(&reply));
        }
    }
    ExitCode::from(super::output::FAILURE)
}
