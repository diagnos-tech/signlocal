//! The framed pipe to one `websign connect` child.

use std::path::Path;
use std::process::{Child, ChildStdin};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use websign_protocol::framing::write_frame;
use websign_protocol::{
    AppEnvelope, ClientEnvelope, ParseError, parse_app_message, parse_hello_reply, to_json,
};

use crate::error::ClientError;
use crate::process;
use crate::reader::{self, Incoming};
use crate::timing::Timing;

/// Which messages the next frame may be.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Expect {
    /// The answer to our `hello`, sent at `hello_v`: the app's `hello`, or
    /// an `error` at `hello_v` refusing ours.
    HelloReply { hello_v: u32 },
    /// Any message at the agreed version.
    Negotiated(u32),
}

impl Expect {
    fn parse(self, frame: &[u8]) -> Result<AppEnvelope, ParseError> {
        match self {
            Expect::HelloReply { hello_v } => parse_hello_reply(frame, hello_v),
            Expect::Negotiated(v) => parse_app_message(frame, Some(v)),
        }
    }
}

/// What waiting for the app produced.
pub(crate) enum Wait {
    Message(Box<AppEnvelope>),
    TimedOut,
    /// The app closed its stdout between two messages.
    Exited,
}

/// A running child, its pipes, and the cleanup that goes with them.
///
/// After any failure the message stream can no longer be trusted (a reply
/// may be missing or late), so the session refuses further use instead of
/// risking answers being paired with the wrong request.
#[derive(Debug)]
pub(crate) struct Session {
    child: Child,
    stdin: Option<ChildStdin>,
    incoming: Receiver<Incoming>,
    usable: bool,
    timing: Timing,
}

impl Session {
    pub(crate) fn start(executable: &Path, timing: Timing) -> Result<Session, ClientError> {
        let spawned = process::spawn(executable, timing.exit_grace)?;
        let mut child = spawned.child;
        match reader::spawn(spawned.stdout) {
            Ok(incoming) => Ok(Session {
                child,
                stdin: Some(spawned.stdin),
                incoming,
                usable: true,
                timing,
            }),
            Err(error) => {
                drop(spawned.stdin);
                process::reap(&mut child, timing.exit_grace);
                Err(ClientError::Connection(format!(
                    "cannot start reader: {error}"
                )))
            }
        }
    }

    pub(crate) fn timing(&self) -> Timing {
        self.timing
    }

    pub(crate) fn send(&mut self, envelope: &ClientEnvelope) -> Result<(), ClientError> {
        self.ensure_usable()?;
        let result = match self.stdin.as_mut() {
            Some(stdin) => write_frame(stdin, &to_json(envelope)),
            None => return Err(self.poison("the pipe to the app is closed")),
        };
        result.map_err(|error| self.poison(format!("cannot write to the app: {error}")))
    }

    /// Waits for the next message. A `timeout` of `None` waits until the app
    /// answers or exits: the person's decision time is enforced by the app.
    pub(crate) fn receive(
        &mut self,
        expect: Expect,
        timeout: Option<Duration>,
    ) -> Result<Wait, ClientError> {
        self.ensure_usable()?;
        let got = match timeout {
            Some(limit) => self.incoming.recv_timeout(limit),
            None => self
                .incoming
                .recv()
                .map_err(|_| RecvTimeoutError::Disconnected),
        };
        match got {
            Ok(Incoming::Frame(frame)) => match expect.parse(&frame) {
                Ok(envelope) => Ok(Wait::Message(Box::new(envelope))),
                Err(error) => Err(self.poison(format!("the app sent an invalid message: {error}"))),
            },
            Ok(Incoming::Broken(why)) => {
                Err(self.poison(format!("cannot read from the app: {why}")))
            }
            Err(RecvTimeoutError::Timeout) => Ok(Wait::TimedOut),
            Err(RecvTimeoutError::Disconnected) => {
                self.usable = false;
                Ok(Wait::Exited)
            }
        }
    }

    /// Marks the session unusable and returns the matching error.
    pub(crate) fn poison(&mut self, why: impl Into<String>) -> ClientError {
        self.usable = false;
        ClientError::Connection(why.into())
    }

    fn ensure_usable(&self) -> Result<(), ClientError> {
        if self.usable {
            Ok(())
        } else {
            Err(ClientError::Connection(
                "the connection to the app is no longer usable".into(),
            ))
        }
    }
}

/// Closing stdin ends the app's session; the child is then reaped, killed
/// after a bounded wait if it lingers.
impl Drop for Session {
    fn drop(&mut self) {
        drop(self.stdin.take());
        process::reap(&mut self.child, self.timing.exit_grace);
    }
}
