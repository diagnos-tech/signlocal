//! What can go wrong, with the protocol's stable codes.

use websign_protocol::ErrorCode;

/// A failed call.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// The `websign` executable was not found or could not start.
    #[error("the WebeSign app is not installed: {0}")]
    AppMissing(String),
    /// The app answered with an error (`UserCancelled`, `PinLocked`, …).
    #[error("{code:?}: {message}")]
    App { code: ErrorCode, message: String },
    /// `prepare` failed; the request was cancelled in the app.
    #[error("prepare failed: {0}")]
    Prepare(String),
    /// The pipe broke or the app sent something unparseable.
    #[error("connection to the app failed: {0}")]
    Connection(String),
}

impl ClientError {
    /// The protocol code for this error (`AppMissing`, `Aborted`, `Internal`
    /// for connection failures).
    pub fn code(&self) -> ErrorCode {
        match self {
            ClientError::AppMissing(_) => ErrorCode::AppMissing,
            ClientError::App { code, .. } => *code,
            ClientError::Prepare(_) => ErrorCode::Aborted,
            ClientError::Connection(_) => ErrorCode::Internal,
        }
    }
}
