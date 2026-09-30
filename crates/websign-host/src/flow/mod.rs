//! Per-request state machines. Pure: each transition returns [`Effect`]s the
//! engine performs (send a message, command the UI, ask the key store).

pub mod choose;
pub mod sign;

use websign_protocol::AppMessage;
use websign_ui_model::confirm::UiCommand;

use crate::ports::KeyCommand;

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
