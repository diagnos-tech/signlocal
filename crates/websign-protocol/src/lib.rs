//! The WebeSign wire contract, shared by every party that talks to the app.
//!
//! One message catalog travels over three framings:
//!
//! * **native messaging** (browser extension ↔ app): 4-byte length + JSON;
//! * **`websign connect`** (desktop program ↔ app): the same framing on the
//!   child process's stdin/stdout;
//! * **page messages** (web page ↔ extension): `window.postMessage`
//!   envelopes around the same messages ([`page`]).
//!
//! Types here are the single source of truth: the TypeScript types of the SDK,
//! the extension and the Node client are generated from them (feature
//! `typescript`), and CI fails when the generated files are stale.
//!
//! This crate only describes and validates messages; it never decides
//! anything. Behavior lives in `websign-host`. The rules are in `SPEC.md`.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod base64;
pub mod code;
pub mod envelope;
pub mod error;
pub mod framing;
pub mod id;
pub mod limits;
pub mod messages;
pub mod page;
mod strict;
pub mod types;
pub mod version;

pub use code::{VerificationCode, verification_code};
pub use envelope::{
    AppEnvelope, ClientEnvelope, ParseError, parse_app_message, parse_client_message,
    parse_hello_reply, refusal_version, to_json,
};
pub use error::{ErrorCode, ErrorDetails, WireError};
pub use id::RequestId;
pub use messages::{AppMessage, ClientMessage};
pub use version::{PROTOCOL_VERSION, ProtocolRange, negotiate};
