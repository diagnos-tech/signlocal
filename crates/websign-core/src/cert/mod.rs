//! Human-meaningful summary of an X.509 certificate.
//!
//! Everything the certificate list and the confirmation window need to show
//! a certificate the way a doctor recognizes it — holder name, ICP-Brasil
//! level, eIDAS qualification, validity — without platform APIs.

mod debug;
mod der;
mod extensions;
mod icp_brasil;
mod key;
mod name_candidate;
mod names;
mod oid;
mod parse;
mod qualified;
mod san;
mod strings;
#[cfg(test)]
mod tests;
mod time;
mod x509;

pub use icp_brasil::{IcpBrasil, IcpLevel};
pub use qualified::{QcType, Qualified};

pub(crate) use key::{PublicKey, RsaComponents};
pub(crate) use parse::decode;

use std::fmt::Display;

use crate::algorithm::SignatureAlgorithm;
use crate::ecdsa::Curve;
use crate::fingerprint::Fingerprint;

/// Summary of one certificate. See `SPEC.md` §6 for every field's rule.
///
/// `Debug` is derived, but the personal identifiers inside it (CPF, subject
/// `serialNumber`) print masked: see `debug.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertInfo {
    pub fingerprint: Fingerprint,
    pub subject: DistinguishedName,
    pub issuer: DistinguishedName,
    /// Lowercase hex of the serial number, without leading zero octets.
    pub serial_hex: String,
    /// Unix seconds; negative before 1970.
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
    ///
    /// The reader is tolerant of encoding slips that do not change the
    /// summary (see `SPEC.md` §6), so a certificate the user owns is not
    /// left out of the list over them.
    pub fn from_der(der: &[u8]) -> Result<CertInfo, CertError> {
        decode(der).map(|(info, _)| info)
    }

    /// Whether `unix_secs` falls inside the validity period (inclusive).
    pub fn is_valid_at(&self, unix_secs: i64) -> bool {
        self.not_before <= unix_secs && unix_secs <= self.not_after
    }

    /// Whether the key may produce signatures (not a CA, usage allows it).
    ///
    /// Extended key usage is deliberately ignored: which usages a document
    /// signature needs is the relying site's decision.
    pub fn can_sign(&self) -> bool {
        !self.is_ca
            && self
                .key_usage
                .is_none_or(|usage| usage.digital_signature || usage.non_repudiation)
    }

    /// The name a person recognizes: [`CertInfo::name_candidate`], or a
    /// fingerprint prefix as the last resort, so the confirmation window never
    /// shows an empty name.
    pub fn display_name(&self) -> String {
        self.name_candidate()
            .unwrap_or_else(|| self.fingerprint.to_hex().chars().take(16).collect())
    }
}

/// Invalid certificate bytes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CertError {
    #[error("malformed certificate: {0}")]
    Malformed(String),
}

/// A `Malformed` error naming the part of the certificate that is broken.
fn malformed(what: &str, detail: impl Display) -> CertError {
    CertError::Malformed(format!("{what}: {detail}"))
}

/// The distinguished-name attributes the UI shows.
///
/// `Debug` masks `serial_number` (a national identifier).
#[derive(Clone, Default, PartialEq, Eq)]
pub struct DistinguishedName {
    pub common_name: Option<String>,
    pub organization: Option<String>,
    pub organizational_units: Vec<String>,
    pub country: Option<String>,
    /// `givenName` (2.5.4.42): the holder-name fallback when CN is absent.
    pub given_name: Option<String>,
    /// `surname` (2.5.4.4).
    pub surname: Option<String>,
    /// `serialNumber` (2.5.4.5): ETSI EN 319 412-1 national identifiers
    /// such as `IDCPT-12345123`. Personal data: only masked forms leave core.
    pub serial_number: Option<String>,
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
        match self {
            Self::Rsa { .. } => algorithm.is_rsa(),
            Self::Ec { .. } => algorithm == SignatureAlgorithm::Ecdsa,
            Self::Unsupported { .. } => false,
        }
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
