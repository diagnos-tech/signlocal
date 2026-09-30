//! The thread that turns the app's output (a child's stdout, or an in-process
//! pipe) into a channel of frames.

use std::io::{self, Read};
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

use websign_protocol::framing::{FrameError, read_frame};

/// Frames buffered before the reader waits for the caller. Bounded so an app
/// that floods stdout cannot make this process allocate without limit.
const BUFFERED_FRAMES: usize = 16;

/// What the reader thread saw on the pipe.
pub(crate) enum Incoming {
    Frame(Vec<u8>),
    /// The stream is unusable (oversized or truncated frame, I/O error).
    Broken(FrameError),
}

/// Reads frames until the stream ends. A clean end of stream simply closes
/// the channel, which the receiver sees as the app having exited.
///
/// A thread is needed because a blocking read cannot time out: without it a
/// silent app would hang `connect` forever. The thread is detached so `Drop`
/// never waits on it: it ends when the pipe closes, which `Session`'s cleanup
/// guarantees by killing the child (unless the app handed its stdout to a
/// process of its own, which then holds one blocked thread and at most one
/// frame until it exits). Once the receiver is gone, `send` fails at once,
/// so a full channel cannot keep it alive either.
pub(crate) fn spawn(mut stdout: impl Read + Send + 'static) -> io::Result<Receiver<Incoming>> {
    let (sender, receiver) = sync_channel(BUFFERED_FRAMES);
    thread::Builder::new()
        .name("websign-client-reader".into())
        .spawn(move || {
            loop {
                let item = match read_frame(&mut stdout) {
                    Ok(Some(frame)) => Incoming::Frame(frame),
                    Ok(None) => return,
                    Err(error) => {
                        let _ = sender.send(Incoming::Broken(error));
                        return;
                    }
                };
                if sender.send(item).is_err() {
                    return;
                }
            }
        })?;
    Ok(receiver)
}
