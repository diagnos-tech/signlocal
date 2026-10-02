//! Per-request state machines. Pure: each transition returns [`Effect`]s the
//! engine performs (send a message, command the UI, ask the key store).

mod certificate;
pub mod choose;
mod context;
mod errors;
mod listing;
mod presentation;
pub mod sign;

use websign_protocol::AppMessage;
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::RequestKey;

use crate::ports::KeyCommand;

pub(crate) use context::accepted_algorithms;
pub use context::list_context;
pub use presentation::Presentation;

/// Something a flow asks the engine to do, in order.
#[derive(Debug)]
pub enum Effect {
    /// Send to the client under the request's id (final messages end the
    /// request).
    Send(AppMessage),
    Ui(UiCommand),
    Keys(KeyCommand),
    /// Remember the caller (ticked "Remember") and/or record certificate use.
    RecordConsent {
        remember: bool,
        fingerprint: String,
    },
    /// Log a failure in the recent-errors store (no personal data).
    RecordError {
        code: websign_protocol::ErrorCode,
        native: Option<String>,
    },
}

/// Tag of the `n`-th key store operation of request `key`. The tag carries
/// its owner, so a reply finds its flow without a lookup table.
pub(crate) fn operation_tag(key: RequestKey, n: u32) -> u64 {
    (key.0 << 32) | u64::from(n)
}

/// The request that started the operation with `tag`.
pub(crate) fn tag_owner(tag: u64) -> RequestKey {
    RequestKey(tag >> 32)
}
