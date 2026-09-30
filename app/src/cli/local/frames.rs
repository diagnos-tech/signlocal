//! The two ends of the in-memory connection: frames into the engine as a
//! byte stream, and replies out of it as protocol messages.

use std::io::Read;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use websign_host::ports::{Outbound, OutboundError};
use websign_protocol::framing::write_frame;
use websign_protocol::messages::{ClientMessage, SignDigest};
use websign_protocol::types::Base64Bytes;
use websign_protocol::{
    AppEnvelope, AppMessage, ClientEnvelope, PROTOCOL_VERSION, RequestId, to_json,
};

/// `message` framed for the engine; `None` if the id is invalid.
pub fn envelope(id: &str, message: ClientMessage) -> Option<Vec<u8>> {
    let envelope = ClientEnvelope {
        v: PROTOCOL_VERSION,
        id: RequestId::new(id).ok()?,
        message,
    };
    let mut frame = Vec::new();
    write_frame(&mut frame, &to_json(&envelope)).ok()?;
    Some(frame)
}

/// The engine's input: framed bytes from a channel; end of stream once
/// every sender is gone.
#[derive(Debug)]
pub struct FrameReader {
    frames: Receiver<Vec<u8>>,
    pending: Vec<u8>,
    at: usize,
}

impl FrameReader {
    /// The sender for frames and the reader the engine consumes.
    pub fn channel() -> (Sender<Vec<u8>>, FrameReader) {
        let (sender, frames) = channel();
        let reader = FrameReader {
            frames,
            pending: Vec::new(),
            at: 0,
        };
        (sender, reader)
    }
}

impl Read for FrameReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while self.at == self.pending.len() {
            match self.frames.recv() {
                Ok(frame) => {
                    self.pending = frame;
                    self.at = 0;
                }
                Err(_) => return Ok(0),
            }
        }
        let n = buf.len().min(self.pending.len() - self.at);
        buf[..n].copy_from_slice(&self.pending[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

/// The client side of the connection: answers `sign.need_digest` with the
/// known digest, keeps the final answer, then closes the stream so the
/// engine ends.
#[derive(Debug)]
pub struct LocalClient {
    to_app: Option<Sender<Vec<u8>>>,
    digest: Option<Vec<u8>>,
    answer: Arc<Mutex<Option<AppMessage>>>,
    request_id: &'static str,
}

impl LocalClient {
    pub fn new(
        to_app: Sender<Vec<u8>>,
        digest: Option<Vec<u8>>,
        answer: Arc<Mutex<Option<AppMessage>>>,
        request_id: &'static str,
    ) -> LocalClient {
        LocalClient {
            to_app: Some(to_app),
            digest,
            answer,
            request_id,
        }
    }

    /// Keeps `message` as the answer and ends the connection.
    fn conclude(&mut self, message: AppMessage) {
        if let Ok(mut slot) = self.answer.lock() {
            slot.get_or_insert(message);
        }
        self.to_app = None;
    }
}

impl Outbound for LocalClient {
    fn send(&mut self, reply: &AppEnvelope) -> Result<(), OutboundError> {
        let ours = reply.id.as_str() == self.request_id;
        match &reply.message {
            AppMessage::Hello(_) => {}
            AppMessage::NeedDigest(need) => {
                let answer = self.digest.clone().map(|digest| {
                    ClientMessage::SignDigest(SignDigest {
                        seq: need.seq,
                        digest: Base64Bytes::new(digest),
                    })
                });
                let frame = answer.and_then(|answer| envelope(self.request_id, answer));
                if let (Some(frame), Some(to_app)) = (frame, &self.to_app) {
                    let _ = to_app.send(frame);
                }
            }
            // An error for `hello` ends the connection as surely as ours.
            AppMessage::Error(_) => self.conclude(reply.message.clone()),
            message if ours && message.is_final() => self.conclude(message.clone()),
            _ => {}
        }
        Ok(())
    }
}
