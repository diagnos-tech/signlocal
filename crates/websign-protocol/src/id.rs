//! Request identifiers.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::limits::MAX_REQUEST_ID_LEN;

/// Identifies one request on one connection; replies and continuation
/// messages (`sign.digest`, `cancel`) carry the id of the request they belong
/// to.
///
/// Chosen by the client. 1 to [`MAX_REQUEST_ID_LEN`] characters from
/// `[A-Za-z0-9._:-]`, so it is safe to log and cannot smuggle markup or
/// control characters into logs or the UI.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export, type = "string"))]
pub struct RequestId(String);

/// A string that is not a valid [`RequestId`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("request id must be 1 to {MAX_REQUEST_ID_LEN} characters from [A-Za-z0-9._:-]")]
pub struct InvalidRequestId;

impl RequestId {
    /// Validates `text` as a request id.
    pub fn new(text: impl Into<String>) -> Result<RequestId, InvalidRequestId> {
        let _ = text.into();
        todo!("SPEC.md §3")
    }

    /// The id as sent on the wire.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RequestId {
    type Error = InvalidRequestId;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        RequestId::new(text)
    }
}

impl From<RequestId> for String {
    fn from(id: RequestId) -> String {
        id.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
