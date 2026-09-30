//! Native messaging wire format: a `u32` length in the machine's own byte
//! order, followed by that many bytes of UTF-8 JSON.
//!
//! The same framing is used by Chrome, Edge, Brave and Firefox in both
//! directions, by `websign connect` on its stdin/stdout, and by the Safari
//! relay socket. Every supported target is little-endian, so desktop clients
//! on any platform write the length little-endian. Only the limits differ per direction, and both are enforced
//! here so that a misbehaving peer can neither make the browser drop the host
//! (oversized replies) nor make the host allocate gigabytes (oversized
//! requests).
//!
//! Promoted from the Phase-0 kit (`probe/src/nm/framing.rs`), reviewed.

use std::io::{self, Read, Write};

/// Largest message the browser accepts from a host. Chrome and Firefox both
/// close the connection when a host exceeds it.
pub const MAX_OUTGOING: usize = 1024 * 1024;

/// Largest request the host accepts. The browsers allow far more, but every
/// message this host understands is tiny, and the buffer is allocated up front.
pub const MAX_INCOMING: usize = 1024 * 1024;

/// Why a frame could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    /// The peer announced (or the caller tried to send) more than the limit.
    #[error("message of {len} bytes exceeds the {max} byte limit")]
    TooLarge { len: usize, max: usize },
    /// The stream ended after part of a header or body: the frame is lost.
    #[error("stream ended in the middle of a message")]
    Truncated,
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// Reads one frame. `Ok(None)` is a clean end of stream, i.e. the browser
/// closed the port between two messages.
pub fn read_frame(reader: &mut impl Read) -> Result<Option<Vec<u8>>, FrameError> {
    let mut header = [0u8; 4];
    let first = read_up_to(reader, &mut header)?;
    if first == 0 {
        return Ok(None);
    }
    if first < header.len() {
        return Err(FrameError::Truncated);
    }
    let len = u32::from_ne_bytes(header) as usize;
    if len > MAX_INCOMING {
        return Err(FrameError::TooLarge {
            len,
            max: MAX_INCOMING,
        });
    }
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            FrameError::Truncated
        } else {
            FrameError::Io(error)
        }
    })?;
    Ok(Some(body))
}

/// Writes one frame and flushes: the browser waits for the whole message
/// before it hands it to the extension.
pub fn write_frame(writer: &mut impl Write, payload: &[u8]) -> Result<(), FrameError> {
    if payload.len() > MAX_OUTGOING {
        return Err(FrameError::TooLarge {
            len: payload.len(),
            max: MAX_OUTGOING,
        });
    }
    // Cannot truncate: `MAX_OUTGOING` fits in a u32.
    let header = (payload.len() as u32).to_ne_bytes();
    writer.write_all(&header)?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

/// Like `read_exact`, but reports how many bytes arrived before the end of
/// the stream instead of failing, so EOF at a frame boundary can be told apart
/// from EOF inside the header.
fn read_up_to(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match reader.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(count) => filled += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(filled)
}

#[cfg(test)]
mod tests;
