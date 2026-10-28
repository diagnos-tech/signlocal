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
use crate::version::PROTOCOL_VERSION;

mod bounds;
mod describe;
mod json;
mod parse;

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
    let checked = parse::check_envelope(frame, negotiated.into(), parse::Sender::Client)?;
    let message = parse::parse_body(&checked)?;
    bounds::check_client(&message).map_err(|why| parse::invalid(Some(&checked.id), why))?;
    Ok(ClientEnvelope {
        v: checked.v,
        id: checked.id,
        message,
    })
}

/// Parses one frame from the app, strictly. Used by client libraries.
///
/// With `None` only the app's `hello` is accepted; to also accept the app
/// refusing a `hello`, use [`parse_hello_reply`].
pub fn parse_app_message(frame: &[u8], negotiated: Option<u32>) -> Result<AppEnvelope, ParseError> {
    parse_app(frame, negotiated.into())
}

/// Parses the app's answer to a `hello` sent with version `hello_v`
/// (`SPEC.md` §5.1): its `hello` (whose `v` is the chosen `protocol`), or an
/// `error` refusing ours, which carries exactly `hello_v`. A refusal is
/// always at a version the client speaks, however far apart the two
/// parties' ranges are.
pub fn parse_hello_reply(frame: &[u8], hello_v: u32) -> Result<AppEnvelope, ParseError> {
    parse_app(frame, parse::Stage::HelloReply { hello_v })
}

/// The `v` of an `error` that answers a connection's first frame, before any
/// version is agreed (`SPEC.md` §5.1): that frame's own `v`, which the
/// client speaks, or [`PROTOCOL_VERSION`] when the frame has no usable `v`
/// (no well-behaved client sends that).
pub fn refusal_version(first_frame: &[u8]) -> u32 {
    json::read(first_frame)
        .and_then(|value| value.get("v")?.as_u64())
        .and_then(|number| u32::try_from(number).ok())
        .filter(|&number| number >= 1)
        .unwrap_or(PROTOCOL_VERSION)
}

fn parse_app(frame: &[u8], stage: parse::Stage) -> Result<AppEnvelope, ParseError> {
    let checked = parse::check_envelope(frame, stage, parse::Sender::App)?;
    let message = parse::parse_body(&checked)?;
    bounds::check_app(&message).map_err(|why| parse::invalid(Some(&checked.id), why))?;
    Ok(AppEnvelope {
        v: checked.v,
        id: checked.id,
        message,
    })
}

/// Serializes an envelope as compact JSON. Infallible for these types.
pub fn to_json<T: Serialize>(envelope: &T) -> Vec<u8> {
    // Every envelope is a struct of strings, numbers and enums with string
    // keys: serialization cannot fail, and an empty frame would be rejected
    // by the peer rather than mistaken for a message.
    serde_json::to_vec(envelope).unwrap_or_default()
}

#[cfg(test)]
mod tests;
