//! The envelope around every message and the strict parser.
//!
//! On the wire a message is one flat JSON object:
//! `{"v": 1, "id": "7", "type": "sign.begin", "hash": "SHA-256", ...}`.
//! Serialization is derived; parsing goes through [`parse_client_message`] /
//! [`parse_app_message`], which enforce what serde's derive cannot combine
//! with a flattened body: unknown fields are refused, and a failure still
//! reports the request id when one was readable, so the error reply reaches
//! the right caller.

use serde::{Deserialize, Serialize};

use crate::error::ErrorCode;
use crate::id::RequestId;
use crate::messages::{AppMessage, ClientMessage};

/// A client message with its envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct ClientEnvelope {
    /// Protocol version of this message's schema: the negotiated version
    /// (for `hello`, the client's highest).
    pub v: u32,
    pub id: RequestId,
    #[serde(flatten)]
    pub message: ClientMessage,
}

/// An app message with its envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct AppEnvelope {
    pub v: u32,
    /// The id of the request this message answers or continues.
    pub id: RequestId,
    #[serde(flatten)]
    pub message: AppMessage,
}

/// A frame that is not a valid message.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{code:?}: {message}")]
pub struct ParseError {
    /// The request id, when the frame had a valid one: the error reply goes
    /// to that request. `None` → the reply cannot be addressed and the
    /// connection is closed after logging.
    pub id: Option<RequestId>,
    /// `InvalidRequest` for every structural problem; `ClientOutdated`/
    /// `AppOutdated` when only the version is wrong.
    pub code: ErrorCode,
    /// English, for developers; names the offending field, never echoes values.
    pub message: String,
}

/// Parses one frame from a client, strictly (`SPEC.md` §5).
///
/// `negotiated` is the version agreed in `hello`, or `None` before it: then
/// only `hello` is accepted.
pub fn parse_client_message(
    frame: &[u8],
    negotiated: Option<u32>,
) -> Result<ClientEnvelope, ParseError> {
    let _ = (frame, negotiated);
    todo!("SPEC.md §5")
}

/// Parses one frame from the app, strictly. Used by client libraries.
pub fn parse_app_message(frame: &[u8], negotiated: Option<u32>) -> Result<AppEnvelope, ParseError> {
    let _ = (frame, negotiated);
    todo!("SPEC.md §5")
}

/// Serializes an envelope as compact JSON. Infallible for these types.
pub fn to_json<T: Serialize>(envelope: &T) -> Vec<u8> {
    let _ = envelope;
    todo!("SPEC.md §5")
}
