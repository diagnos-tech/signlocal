//! Frames over the process's stdin/stdout.

use std::io::Stdout;

use websign_protocol::AppEnvelope;

use super::EventSender;
use crate::ports::{Outbound, OutboundError};

/// Writes frames to stdout. stdout carries frames only: logs go to the log
/// file, never to stdout.
#[derive(Debug)]
pub struct StdoutOutbound {
    stdout: Stdout,
}

impl StdoutOutbound {
    /// Takes over stdout.
    pub fn new() -> StdoutOutbound {
        StdoutOutbound {
            stdout: std::io::stdout(),
        }
    }
}

impl Default for StdoutOutbound {
    fn default() -> Self {
        StdoutOutbound::new()
    }
}

impl Outbound for StdoutOutbound {
    fn send(&mut self, envelope: &AppEnvelope) -> Result<(), OutboundError> {
        let _ = (envelope, &self.stdout);
        todo!("SPEC.md §9")
    }
}

/// Reads frames from stdin on a new thread and posts them as
/// `Frame`/`Closed`/`Broken` events.
pub fn spawn_stdin_reader(events: EventSender) -> std::thread::JoinHandle<()> {
    let _ = events;
    todo!("SPEC.md §9")
}
