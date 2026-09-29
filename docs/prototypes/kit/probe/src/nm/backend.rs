//! What the protocol handler needs from the machine's key sources.
//!
//! A trait, so the handler's rules (validation order, error codes, reply
//! shapes) are tested without touching a keystore.

use probe_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm};
use serde::Serialize;

use super::protocol::ProtocolError;

/// One certificate as the extension sees it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CertificateSummary {
    /// SHA-256 of the DER, 64 lowercase hex digits. Selects the key in `sign`.
    pub fingerprint: String,
    pub display_name: String,
    /// `icp-brasil`, `eidas-qualified` or `certificate`.
    pub kind: String,
    /// ICP-Brasil level (`A3`), when known.
    pub level: Option<String>,
    /// `RSA-2048`, `EC P-256`, ...
    pub key: String,
    /// Key source, e.g. `windows` or `pkcs11:libsofthsm2.so`.
    pub origin: String,
    /// Where the key lives, for people: provider or token label.
    pub provider: String,
    pub hardware: Option<bool>,
    /// Who asks for the PIN: `system`, `app` or `pin-pad`.
    pub pin: &'static str,
    pub can_sign: bool,
    /// WebCrypto names of the signature algorithms the key supports.
    pub algorithms: Vec<&'static str>,
    /// Unix seconds.
    pub not_before: i64,
    /// Unix seconds.
    pub not_after: i64,
    /// How many key sources expose this same certificate.
    pub paths: usize,
}

/// Every certificate found, plus sources that could not be read.
#[derive(Debug, Default)]
pub struct CertificateList {
    pub certificates: Vec<CertificateSummary>,
    /// One line per source that failed to open or list. Not personal data.
    pub warnings: Vec<String>,
}

/// A validated signing request: every field is already the right type and
/// the digest already has the exact length of `hash`.
#[derive(Debug)]
pub struct SignJob {
    pub fingerprint: Fingerprint,
    pub hash: HashAlgorithm,
    pub algorithm: SignatureAlgorithm,
    pub digest: Vec<u8>,
    /// Window that should own OS PIN dialogs (Windows).
    pub parent_window: Option<isize>,
}

/// A finished signature.
#[derive(Debug)]
pub struct SignedDigest {
    /// Final format: RSA block, or ECDSA raw `r || s`.
    pub signature: Vec<u8>,
    /// Native call that produced it, e.g. `C_Sign`.
    pub api: &'static str,
    pub elapsed_ms: u64,
    /// The host checked the signature against the certificate's public key.
    pub verified: bool,
}

pub trait Backend {
    /// Lists signing certificates, loading key sources on first use.
    fn certificates(&mut self) -> Result<CertificateList, ProtocolError>;

    fn sign(&mut self, job: &SignJob) -> Result<SignedDigest, ProtocolError>;
}
