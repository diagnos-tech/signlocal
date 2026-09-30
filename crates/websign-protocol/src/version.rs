//! Protocol versions and their negotiation.
//!
//! A protocol version is one integer. Any change to the message catalog, even
//! an added optional field, is a new version: messages are parsed strictly
//! (unknown fields are refused), so peers must agree on the exact schema.
//! Each side supports a contiguous range and `hello` picks the highest common
//! version, so an old extension keeps working with a newer app and vice versa
//! for as long as the ranges overlap.

use serde::{Deserialize, Serialize};

use crate::error::ErrorCode;

/// The newest protocol version this crate describes.
pub const PROTOCOL_VERSION: u32 = 1;

/// The oldest protocol version this crate can still parse.
pub const OLDEST_SUPPORTED_VERSION: u32 = 1;

/// A contiguous, inclusive range of protocol versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct ProtocolRange {
    pub min: u32,
    pub max: u32,
}

impl ProtocolRange {
    /// The range this build of the protocol crate speaks.
    pub const CURRENT: ProtocolRange = ProtocolRange {
        min: OLDEST_SUPPORTED_VERSION,
        max: PROTOCOL_VERSION,
    };
}

/// Picks the version both sides speak, or says which side is too old.
///
/// * overlap → `Ok(highest common version)`;
/// * `client.max < app.min` → `Err(ErrorCode::ClientOutdated)` (the web
///   transport reports it to pages as `ExtensionOutdated`);
/// * `app.max < client.min` → `Err(ErrorCode::AppOutdated)`;
/// * a malformed range (`min > max` or `min == 0`) → `Err(ErrorCode::InvalidRequest)`.
pub fn negotiate(app: ProtocolRange, client: ProtocolRange) -> Result<u32, ErrorCode> {
    let _ = (app, client);
    todo!("SPEC.md §2")
}
