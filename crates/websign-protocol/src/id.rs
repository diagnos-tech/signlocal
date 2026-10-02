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
        let text = text.into();
        let valid_len = (1..=MAX_REQUEST_ID_LEN).contains(&text.len());
        if valid_len && text.bytes().all(is_id_byte) {
            Ok(RequestId(text))
        } else {
            Err(InvalidRequestId)
        }
    }

    /// The id as sent on the wire.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_id_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_documented_alphabet_and_lengths() {
        for id in ["1", "h", "p1.3.abc", "a:b-c_d", &"a".repeat(64)] {
            assert_eq!(
                RequestId::new(id).map(|i| i.to_string()),
                Ok(id.to_string())
            );
        }
    }

    #[test]
    fn rejects_everything_else() {
        for id in ["", &"a".repeat(65), "a b", "é", "a\n", "<x>"] {
            assert_eq!(RequestId::new(id), Err(InvalidRequestId), "{id:?}");
        }
    }

    #[test]
    fn serde_applies_the_same_check() {
        assert!(serde_json::from_str::<RequestId>("\"ok.1\"").is_ok());
        assert!(serde_json::from_str::<RequestId>("\"no way\"").is_err());
        assert!(serde_json::from_str::<RequestId>("7").is_err());
    }
}
