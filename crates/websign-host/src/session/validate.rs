//! Transport-specific rules on top of strict parsing (`SPEC.md` §2.2).

use websign_protocol::ClientMessage;
use websign_protocol::WireError;

use super::Transport;

/// Checks `message` against the transport: over native messaging, `hello`
/// needs `browser` and `status`/`choose`/`sign.begin` need `web`; over
/// `websign connect`, both are refused; `diagnostics.open` is accepted on both
/// (the extension never forwards it from pages).
pub fn validate(transport: &Transport, message: &ClientMessage) -> Result<(), WireError> {
    let _ = (transport, message);
    todo!("SPEC.md §2.2")
}
