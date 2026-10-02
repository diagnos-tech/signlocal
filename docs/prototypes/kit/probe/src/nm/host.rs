//! The native messaging loop: read a request frame, answer with a reply frame,
//! until the browser closes the port.

use std::io::{self, Read, Write};
use std::process::ExitCode;
use std::time::Instant;

use super::backend::Backend;
use super::framing::{FrameError, read_frame, write_frame};
use super::handler::{Handler, Reply, refuse};
use super::keystore_backend::KeystoreBackend;
use super::launch::BrowserLaunch;
use super::log::{HostLog, log_path};
use super::protocol::{ErrorCode, ProtocolError};

/// Arguments of `websign-probe host`.
///
/// Runs the very same loop a browser would start, but on your terminal's
/// stdin/stdout, to exercise the host outside a browser (pipe framed
/// messages in). Reports where the log goes on stderr; stdout carries frames
/// only.
#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let Args {} = args;
    eprintln!("host log: {}", log_path().display());
    Ok(serve_stdio(&BrowserLaunch::manual()))
}

/// Serves native messaging on stdin/stdout until the browser disconnects.
pub fn serve_stdio(launch: &BrowserLaunch) -> ExitCode {
    let log = HostLog::open(launch);
    log.started(launch.parent_window.is_some());
    let mut handler = Handler::new(launch.clone(), KeystoreBackend::new(), log.clone());
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    match serve(&mut input, &mut output, &mut handler, &log) {
        Ended::Disconnected => ExitCode::SUCCESS,
        Ended::Failed => ExitCode::FAILURE,
    }
}

/// How the loop ended.
#[derive(Debug, PartialEq, Eq)]
pub enum Ended {
    /// The browser closed the port between two messages: the normal way out.
    Disconnected,
    /// The stream broke or violated the framing.
    Failed,
}

/// The message loop, over any byte streams.
pub fn serve<B: Backend>(
    input: &mut impl Read,
    output: &mut impl Write,
    handler: &mut Handler<B>,
    log: &HostLog,
) -> Ended {
    loop {
        let started = Instant::now();
        let reply = match read_frame(input) {
            Ok(Some(frame)) => handler.handle(&frame),
            Ok(None) => {
                log.stopped("eof");
                return Ended::Disconnected;
            }
            Err(FrameError::TooLarge { len, max }) => {
                // The body was not consumed, so the stream is out of step:
                // say why, then stop instead of parsing garbage as a header.
                let error = ProtocolError::new(
                    ErrorCode::BadRequest,
                    format!("message of {len} bytes exceeds the {max} byte limit"),
                );
                let _ = send(output, &refuse(&error), log, started);
                log.stopped("message-too-large");
                return Ended::Failed;
            }
            Err(FrameError::Truncated) => {
                log.stopped("truncated");
                return Ended::Failed;
            }
            Err(FrameError::Io(_)) => {
                log.stopped("read-error");
                return Ended::Failed;
            }
        };
        if send(output, &reply, log, started).is_err() {
            log.stopped("write-error");
            return Ended::Failed;
        }
    }
}

/// Serializes and writes `reply`, replacing it with an `internal` error when
/// it would exceed the browser's size limit.
fn send(
    output: &mut impl Write,
    reply: &Reply,
    log: &HostLog,
    started: Instant,
) -> Result<(), FrameError> {
    let bytes = serde_json::to_vec(&reply.message).map_err(io::Error::from)?;
    match write_frame(output, &bytes) {
        Err(FrameError::TooLarge { .. }) => {
            let error = ProtocolError::new(ErrorCode::Internal, "reply exceeds the size limit");
            let fallback = refuse(&error);
            let bytes = serde_json::to_vec(&fallback.message).map_err(io::Error::from)?;
            log.reply(fallback.outcome, bytes.len(), started.elapsed());
            write_frame(output, &bytes)
        }
        other => {
            log.reply(reply.outcome, bytes.len(), started.elapsed());
            other
        }
    }
}

#[cfg(test)]
mod tests;
