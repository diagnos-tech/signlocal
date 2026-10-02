//! Frames over the process's stdin/stdout.

use std::io::{Read, Stdout, Write};

use websign_protocol::framing::{FrameError, read_frame, write_frame};
use websign_protocol::limits::MAX_OUTGOING_FRAME;
use websign_protocol::{AppEnvelope, AppMessage, ErrorCode, WireError, to_json};

use super::EventSender;
use crate::engine::EngineEvent;
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
        write_reply(&mut self.stdout.lock(), envelope)
    }
}

/// Frames to any writer (a pipe in tests, an accepted socket later).
#[derive(Debug)]
pub struct WriterOutbound<W: Write> {
    writer: W,
}

impl<W: Write> WriterOutbound<W> {
    pub fn new(writer: W) -> WriterOutbound<W> {
        WriterOutbound { writer }
    }
}

impl<W: Write> Outbound for WriterOutbound<W> {
    fn send(&mut self, envelope: &AppEnvelope) -> Result<(), OutboundError> {
        write_reply(&mut self.writer, envelope)
    }
}

/// Serializes and writes one reply. A reply over the frame limit would make
/// the browser drop the whole host, so the caller gets an `Internal` error
/// for the same request instead.
fn write_reply(writer: &mut impl Write, envelope: &AppEnvelope) -> Result<(), OutboundError> {
    let mut bytes = to_json(envelope);
    if bytes.len() > MAX_OUTGOING_FRAME {
        log::warn!("a reply of {} bytes was replaced by an error", bytes.len());
        bytes = to_json(&AppEnvelope {
            v: envelope.v,
            id: envelope.id.clone(),
            message: AppMessage::Error(WireError {
                code: ErrorCode::Internal,
                message: "the reply is too large to send".to_owned(),
                details: None,
            }),
        });
    }
    write_frame(writer, &bytes).map_err(|error| OutboundError(error.to_string()))
}

/// Reads frames from stdin on a new thread and posts them as
/// `Frame`/`Closed`/`Broken` events.
pub fn spawn_stdin_reader(events: EventSender) -> std::thread::JoinHandle<()> {
    spawn_reader(std::io::stdin(), events)
}

/// [`spawn_stdin_reader`] over any reader. The thread ends at the end of the
/// stream, at the first framing error, or when the engine stops listening.
pub fn spawn_reader(
    mut input: impl Read + Send + 'static,
    events: EventSender,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            let event = match read_frame(&mut input) {
                Ok(Some(frame)) => EngineEvent::Frame(frame),
                Ok(None) => EngineEvent::Closed,
                Err(error) => EngineEvent::Broken(describe(&error)),
            };
            let last = !matches!(event, EngineEvent::Frame(_));
            if events.send(event).is_err() || last {
                break;
            }
        }
    })
}

/// The reason for the log: sizes and kinds, never content.
fn describe(error: &FrameError) -> String {
    match error {
        FrameError::TooLarge { len, max } => format!("frame of {len} bytes exceeds {max}"),
        FrameError::Truncated => "stream ended inside a frame".to_owned(),
        FrameError::Io(error) => format!("read failed: {:?}", error.kind()),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::mpsc;

    use websign_protocol::RequestId;
    use websign_protocol::messages::Done;

    use super::*;

    fn frames(bytes: Vec<u8>) -> Vec<EngineEvent> {
        let (sender, receiver) = mpsc::channel();
        spawn_reader(Cursor::new(bytes), sender).join().unwrap();
        receiver.try_iter().collect()
    }

    fn framed(payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        write_frame(&mut out, payload).unwrap();
        out
    }

    #[test]
    fn frames_then_a_clean_end_become_frame_and_closed_events() {
        let mut input = framed(b"one");
        input.extend(framed(b"two"));
        let events = frames(input);
        assert!(matches!(&events[0], EngineEvent::Frame(f) if f == b"one"));
        assert!(matches!(&events[1], EngineEvent::Frame(f) if f == b"two"));
        assert!(matches!(events[2], EngineEvent::Closed));
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn a_truncated_frame_is_reported_once_and_stops_the_reader() {
        let mut input = framed(b"complete");
        input.extend_from_slice(&[10, 0, 0, 0, b'x']);
        let events = frames(input);
        assert!(matches!(events.last(), Some(EngineEvent::Broken(_))));
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn an_oversized_announcement_is_broken_without_allocating() {
        let events = frames(u32::MAX.to_ne_bytes().to_vec());
        assert!(matches!(&events[..], [EngineEvent::Broken(reason)] if reason.contains("exceeds")));
    }

    fn envelope(message: AppMessage) -> AppEnvelope {
        AppEnvelope {
            v: 1,
            id: RequestId::new("7").unwrap(),
            message,
        }
    }

    #[test]
    fn replies_are_written_as_frames() {
        let mut out = WriterOutbound::new(Vec::new());
        out.send(&envelope(AppMessage::Done(Done {}))).unwrap();
        let mut cursor = Cursor::new(out.writer);
        let frame = read_frame(&mut cursor).unwrap().unwrap();
        assert_eq!(
            String::from_utf8(frame).unwrap(),
            r#"{"v":1,"id":"7","type":"done"}"#
        );
    }

    #[test]
    fn an_oversized_reply_becomes_an_internal_error_for_the_same_request() {
        let huge = envelope(AppMessage::Error(WireError {
            code: ErrorCode::DriverFailure,
            message: "x".repeat(MAX_OUTGOING_FRAME),
            details: None,
        }));
        let mut written = Vec::new();
        write_reply(&mut written, &huge).unwrap();
        let frame = read_frame(&mut Cursor::new(written)).unwrap().unwrap();
        let text = String::from_utf8(frame).unwrap();
        assert!(
            text.contains(r#""id":"7""#) && text.contains("Internal"),
            "{text}"
        );
    }

    #[test]
    fn a_closed_reader_is_a_delivery_error() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut out = WriterOutbound::new(Broken);
        assert!(out.send(&envelope(AppMessage::Done(Done {}))).is_err());
    }
}
