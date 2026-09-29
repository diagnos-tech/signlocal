//! Human-meaningful summary of an X.509 certificate.
//!
//! Everything the certificate list and the confirmation window need to show
//! a certificate the way a doctor recognizes it — holder name, ICP-Brasil
//! level, eIDAS qualification, validity — without platform APIs.

mod icp_brasil;
mod qualified;

pub use icp_brasil::{IcpBrasil, IcpLevel};
pub use qualified::{QcType, Qualified};

use crate::algorithm::SignatureAlgorithm;
use crate::ecdsa::Curve;
use crate::fingerprint::Fingerprint;

/// Summary of one certificate. See `SPEC.md` §6 for every field's rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertInfo {
    pub fingerprint: Fingerprint,
    pub subject: DistinguishedName,
    pub issuer: DistinguishedName,
    /// Lowercase hex of the serial number, without a leading sign byte.
    pub serial_hex: String,
    /// Unix seconds.
    pub not_before: i64,
    /// Unix seconds.
    pub not_after: i64,
    pub key: PublicKeyKind,
    /// `None` when the certificate has no KeyUsage extension.
    pub key_usage: Option<KeyUsage>,
    /// Dotted OIDs, in certificate order.
    pub extended_key_usage: Vec<String>,
    /// Dotted certificate policy OIDs, in certificate order.
    pub policies: Vec<String>,
    pub is_ca: bool,
    pub icp_brasil: Option<IcpBrasil>,
    pub qualified: Option<Qualified>,
}

impl CertInfo {
    /// Parses a DER certificate.
    pub fn from_der(der: &[u8]) -> Result<CertInfo, CertError> {
        todo!()
    }

    /// Whether `unix_secs` falls inside the validity period (inclusive).
    pub fn is_valid_at(&self, unix_secs: i64) -> bool {
        todo!()
    }

    /// Whether the key may produce signatures (not a CA, usage allows it).
    ///
    /// Extended key usage is deliberately ignored: which usages a document
    /// signature needs is the relying site's decision.
    pub fn can_sign(&self) -> bool {
        todo!()
    }

    /// The name a person recognizes: ICP-Brasil holder, CN, O, or a
    /// fingerprint prefix as the last resort.
    pub fn display_name(&self) -> String {
        todo!()
    }
}

/// Invalid certificate bytes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CertError {
    #[error("malformed certificate: {0}")]
    Malformed(String),
}

/// The distinguished-name attributes the UI shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DistinguishedName {
    pub common_name: Option<String>,
    pub organization: Option<String>,
    pub organizational_units: Vec<String>,
    pub country: Option<String>,
}

/// Public key type, as far as signing is concerned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicKeyKind {
    Rsa {
        bits: u32,
    },
    Ec {
        curve: Curve,
    },
    /// Any other algorithm or curve, identified by its dotted OID.
    Unsupported {
        oid: String,
    },
}

impl PublicKeyKind {
    /// Whether `algorithm` can be used with this key.
    pub fn supports(&self, algorithm: SignatureAlgorithm) -> bool {
        todo!()
    }
}

/// KeyUsage extension bits (RFC 5280 §4.2.1.3).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyUsage {
    pub digital_signature: bool,
    /// Also known as `contentCommitment`.
    pub non_repudiation: bool,
    pub key_encipherment: bool,
    pub data_encipherment: bool,
    pub key_agreement: bool,
    pub key_cert_sign: bool,
    pub crl_sign: bool,
}
