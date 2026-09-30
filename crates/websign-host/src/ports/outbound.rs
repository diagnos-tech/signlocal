//! Sending messages to the client.

use websign_protocol::AppEnvelope;

/// The transport could not deliver: the client is gone.
#[derive(Debug, thiserror::Error)]
#[error("cannot write to the client: {0}")]
pub struct OutboundError(pub String);

/// Writes one framed message. Implementations enforce
/// `websign_protocol::limits::MAX_OUTGOING_FRAME` (an oversized reply
/// becomes an `Internal` error reply, as the kit did).
pub trait Outbound {
    fn send(&mut self, envelope: &AppEnvelope) -> Result<(), OutboundError>;
}
