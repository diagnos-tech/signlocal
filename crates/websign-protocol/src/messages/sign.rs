//! The signing exchange: `sign.begin` → (`sign.need_digest` ↔ `sign.digest`)+ → `sign.result`.

use serde::{Deserialize, Serialize};

use crate::types::{
    Base64Bytes, Certificate, FingerprintHex, HashName, SignatureAlgorithmName, WebContext,
};

/// Starts a signature. Opens the confirmation window; the certificate is
/// chosen there, and the digest is asked for once it is known, because
/// formats such as PAdES put the certificate inside the signed data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct SignBegin {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub web: Option<WebContext>,
    /// The hash the caller will compute the digest with.
    pub hash: HashName,
    /// Acceptable algorithms, preferred first. Absent =
    /// [`SignatureAlgorithmName::DEFAULT_PREFERENCE`]. Certificates whose key
    /// can do none of them are shown disabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub algorithms: Option<Vec<SignatureAlgorithmName>>,
    /// Preselect this certificate (e.g. from an earlier `choose`). Unknown or
    /// unusable → ignored; the person chooses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub certificate: Option<FingerprintHex>,
}

/// Asks the caller for the digest to sign with `certificate`.
///
/// Sent when a certificate is released to the caller: at once for a
/// remembered caller's preselection, otherwise after the person confirms the
/// certificate (`docs/plan.md` D11). Sent again with `seq + 1` whenever the
/// person switches certificate; digests for an older `seq` are ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct NeedDigest {
    /// Starts at 1 for each request; strictly increasing.
    pub seq: u32,
    pub certificate: Certificate,
    pub hash: HashName,
    /// The algorithm the signature will use (it goes into CMS
    /// `signatureAlgorithm` and the algorithm-protection attribute).
    pub algorithm: SignatureAlgorithmName,
}

/// The digest for `sign.need_digest` number `seq`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct SignDigest {
    pub seq: u32,
    /// Exactly `hash.digest_len()` bytes, else the request fails with
    /// `InvalidRequest`.
    pub digest: Base64Bytes,
}

/// The signature, ready to embed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct SignResult {
    pub certificate: Certificate,
    pub hash: HashName,
    pub algorithm: SignatureAlgorithmName,
    /// RSA: the signature block (modulus length). ECDSA: raw `r || s`, each
    /// half left-padded to the curve size (IEEE P1363).
    pub signature: Base64Bytes,
}
