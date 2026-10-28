//! Binary data on the wire.

use serde::{Deserialize, Serialize};

/// Bytes carried as canonical padded standard Base64 (RFC 4648 §4).
///
/// Decoding is strict ([`crate::base64::decode`]): a digest that could decode
/// two ways is refused rather than guessed.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export, type = "string"))]
pub struct Base64Bytes(Vec<u8>);

impl Base64Bytes {
    /// Wraps raw bytes.
    pub fn new(bytes: Vec<u8>) -> Base64Bytes {
        Base64Bytes(bytes)
    }

    /// The decoded bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Unwraps the decoded bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl TryFrom<String> for Base64Bytes {
    type Error = crate::base64::DecodeError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        crate::base64::decode(&text).map(Base64Bytes)
    }
}

impl From<Base64Bytes> for String {
    fn from(bytes: Base64Bytes) -> String {
        crate::base64::encode(&bytes.0)
    }
}

/// Only the length: digests and certificates must not end up in logs.
impl std::fmt::Debug for Base64Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Base64Bytes({} bytes)", self.0.len())
    }
}
