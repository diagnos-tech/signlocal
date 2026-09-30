//! The message catalog of protocol version 1.
//!
//! A *request* (`hello`, `status`, `choose`, `sign.begin`,
//! `diagnostics.open`) gets exactly one *final* reply carrying its id:
//! the matching result, `done`, or `error`. `sign.need_digest` is the only
//! non-final message: the app sends it (possibly several times) while a
//! `sign.begin` is open, and the client answers each with `sign.digest`.
//! `sign.digest` and `cancel` are *continuations*: they reuse the id of an open
//! request and never get a reply of their own.
//!
//! `docs/architecture/protocol.md` has JSON examples and the state machine.

mod control;
mod hello;
mod sign;
mod status;

use serde::{Deserialize, Serialize};

pub use control::{Cancel, DiagnosticsTab, Done, OpenDiagnostics};
pub use hello::{Hello, HelloReply};
pub use sign::{NeedDigest, SignBegin, SignDigest, SignResult};
pub use status::{Choose, ChooseResult, Status, StatusReply};

use crate::error::WireError;

/// Client → app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum ClientMessage {
    #[serde(rename = "hello")]
    Hello(Hello),
    #[serde(rename = "status")]
    Status(Status),
    #[serde(rename = "choose")]
    Choose(Choose),
    #[serde(rename = "sign.begin")]
    SignBegin(SignBegin),
    #[serde(rename = "sign.digest")]
    SignDigest(SignDigest),
    #[serde(rename = "cancel")]
    Cancel(Cancel),
    #[serde(rename = "diagnostics.open")]
    OpenDiagnostics(OpenDiagnostics),
}

/// App → client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum AppMessage {
    #[serde(rename = "hello")]
    Hello(HelloReply),
    #[serde(rename = "status")]
    Status(StatusReply),
    #[serde(rename = "choose.result")]
    ChooseResult(ChooseResult),
    #[serde(rename = "sign.need_digest")]
    NeedDigest(NeedDigest),
    #[serde(rename = "sign.result")]
    SignResult(SignResult),
    #[serde(rename = "done")]
    Done(Done),
    #[serde(rename = "error")]
    Error(WireError),
}

impl ClientMessage {
    /// The `"type"` string; a fixed set, safe to log.
    pub const fn kind(&self) -> &'static str {
        match self {
            ClientMessage::Hello(_) => "hello",
            ClientMessage::Status(_) => "status",
            ClientMessage::Choose(_) => "choose",
            ClientMessage::SignBegin(_) => "sign.begin",
            ClientMessage::SignDigest(_) => "sign.digest",
            ClientMessage::Cancel(_) => "cancel",
            ClientMessage::OpenDiagnostics(_) => "diagnostics.open",
        }
    }

    /// Whether this message continues an open request instead of starting one.
    pub const fn is_continuation(&self) -> bool {
        matches!(
            self,
            ClientMessage::SignDigest(_) | ClientMessage::Cancel(_)
        )
    }
}

impl AppMessage {
    /// Whether this message ends its request (everything but `sign.need_digest`).
    pub const fn is_final(&self) -> bool {
        !matches!(self, AppMessage::NeedDigest(_))
    }
}
