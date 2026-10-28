//! A certificate as callers receive it.

use serde::{Deserialize, Serialize};

use super::{Base64Bytes, CurveName, SignatureAlgorithmName};

/// SHA-256 of a certificate's DER, as 64 lowercase hex digits: the identity of
/// a certificate everywhere in the project.
///
/// `Debug` prints only the first 8 digits: the full value identifies one
/// person's certificate and must not reach logs.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export, type = "string"))]
pub struct FingerprintHex(String);

impl std::fmt::Debug for FingerprintHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = self.0.get(..8).unwrap_or(&self.0);
        write!(f, "FingerprintHex({prefix}…)")
    }
}

/// Not 64 lowercase hex digits.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("fingerprint must be 64 lowercase hex digits")]
pub struct InvalidFingerprint;

impl FingerprintHex {
    /// Validates `text`: exactly 64 characters from `[0-9a-f]`.
    pub fn new(text: impl Into<String>) -> Result<FingerprintHex, InvalidFingerprint> {
        let text = text.into();
        let valid =
            text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        if valid {
            Ok(FingerprintHex(text))
        } else {
            Err(InvalidFingerprint)
        }
    }

    /// The hex text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FingerprintHex {
    type Error = InvalidFingerprint;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        FingerprintHex::new(text)
    }
}

impl From<FingerprintHex> for String {
    fn from(fingerprint: FingerprintHex) -> String {
        fingerprint.0
    }
}

/// A certificate the person chose (never the machine's list, `docs/plan.md` D2).
///
/// `Debug` is written by hand and prints only non-personal facts: the holder's
/// name, the issuer, the DER and the chain would put personal data in logs.
///
/// Everything a signature format needs to be assembled by the caller, plus a
/// profile so the caller can enforce its own "qualified" policy. The app never
/// claims legal qualification; it reports what the certificate says.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct Certificate {
    /// The certificate, DER.
    pub der: Base64Bytes,
    /// Issuer certificates, leaf excluded, nearest first; best effort (OS
    /// chain engine or certificates on the token). May be empty; at most 8
    /// (`MAX_CHAIN_LEN`).
    pub chain: Vec<Base64Bytes>,
    pub fingerprint: FingerprintHex,
    /// The holder's name as the window shows it (`"Ana Beatriz Souza"`).
    pub display_name: String,
    /// Issuer CN, else O.
    pub issuer_name: String,
    /// Unix seconds.
    #[cfg_attr(feature = "typescript", ts(type = "number"))]
    pub not_before: i64,
    /// Unix seconds.
    #[cfg_attr(feature = "typescript", ts(type = "number"))]
    pub not_after: i64,
    pub key: KeyDescription,
    /// Algorithms this key can produce through the key store that holds it.
    pub algorithms: Vec<SignatureAlgorithmName>,
    pub profile: CertificateProfile,
}

impl std::fmt::Debug for Certificate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fingerprint = self.fingerprint.as_str();
        f.debug_struct("Certificate")
            .field("fingerprint", &format_args!("{}...", &fingerprint[..8]))
            .field("key", &self.key)
            .field("not_before", &self.not_before)
            .field("not_after", &self.not_after)
            .field("algorithms", &self.algorithms)
            .field("profile", &self.profile)
            .field("chain_len", &self.chain.len())
            .finish_non_exhaustive()
    }
}

/// The public key, as far as a caller building a signature format cares.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", tag = "type", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum KeyDescription {
    #[serde(rename = "RSA")]
    Rsa { bits: u32 },
    #[serde(rename = "EC")]
    Ec { curve: CurveName },
}

/// What the certificate declares about its legal profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct CertificateProfile {
    /// ICP-Brasil class from the certificate policy (`"A1"`, `"A3"`, `"S3"`,
    /// `"T3"`, …); `"ICP-Brasil"` when it is ICP-Brasil with an unknown
    /// class; absent when not ICP-Brasil.
    #[serde(
        default,
        deserialize_with = "crate::strict::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub icp_brasil: Option<String>,
    /// Present when the certificate has ETSI qcStatements.
    #[serde(
        default,
        deserialize_with = "crate::strict::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub eidas: Option<EidasProfile>,
    /// Where the private key lives, as far as the key store can tell.
    pub key_storage: KeyStorage,
}

/// ETSI EN 319 412-5 statements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct EidasProfile {
    /// `QcCompliance`: issued as a qualified certificate.
    pub qualified: bool,
    /// `QcSSCD`: the key is declared to be in a qualified signature creation device.
    pub qscd: bool,
    /// `QcType` values, in certificate order.
    pub types: Vec<EidasType>,
}

/// `QcType` statement values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum EidasType {
    Esign,
    Eseal,
    Web,
}

/// Whether the private key is in hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum KeyStorage {
    Hardware,
    Software,
    Unknown,
}

/// Which certificates a request can use (`docs/ux.md` R6). Others are shown
/// disabled as "Not compatible with this request".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct CertificateFilter {
    /// Only keys that can produce one of these. Absent = any; never empty.
    #[serde(
        default,
        deserialize_with = "crate::strict::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub algorithms: Option<Vec<SignatureAlgorithmName>>,
}

crate::strict::object_serde!(
    Certificate,
    KeyDescription,
    CertificateProfile,
    EidasProfile,
    CertificateFilter
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_prints_identity_fields() {
        let certificate = Certificate {
            der: Base64Bytes::new(b"CPF 123.456.789-09".to_vec()),
            chain: vec![Base64Bytes::new(vec![1])],
            fingerprint: FingerprintHex::new("ab".repeat(32)).unwrap(),
            display_name: "Ana Beatriz Souza 123.456.789-09".into(),
            issuer_name: "AC Example".into(),
            not_before: 1,
            not_after: 2,
            key: KeyDescription::Rsa { bits: 2048 },
            algorithms: vec![],
            profile: CertificateProfile {
                icp_brasil: None,
                eidas: None,
                key_storage: KeyStorage::Hardware,
            },
        };
        let text = format!("{certificate:?} {certificate:#?}");
        for secret in ["Ana", "Souza", "123.456", "AC Example", &"ab".repeat(32)] {
            assert!(!text.contains(secret), "{secret} leaked: {text}");
        }
        assert!(text.contains("abababab..."));
        assert!(text.contains("chain_len: 1"));
    }

    #[test]
    fn accepts_64_lowercase_hex_digits() {
        assert!(FingerprintHex::new("0123456789abcdef".repeat(4)).is_ok());
    }

    #[test]
    fn rejects_uppercase_separators_and_wrong_lengths() {
        let ok = "ab".repeat(32);
        assert!(FingerprintHex::new(ok.to_uppercase()).is_err());
        assert!(FingerprintHex::new(&ok[..63]).is_err());
        assert!(FingerprintHex::new(format!("{ok}0")).is_err());
        assert!(FingerprintHex::new(format!("{}:{}", &ok[..31], &ok[32..])).is_err());
        assert!(FingerprintHex::new("é".repeat(32)).is_err());
    }
}
