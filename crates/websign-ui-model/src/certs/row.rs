//! A row of the list, and the list itself.

use websign_core::Fingerprint;

use super::badge::Badge;
use super::candidate::CertCandidate;
use super::location::Location;
use super::validity::ValidityLabel;

/// One visible certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertRow {
    pub candidate: CertCandidate,
    /// Line 1: title-cased holder name (`websign_core::present::holder`).
    pub name: String,
    pub badge: Badge,
    /// Line 2: masked document, if any.
    pub document: Option<websign_core::present::document::DocumentLabel>,
    /// Line 2: short issuer (CN, else O).
    pub issuer: String,
    /// Line 3.
    pub location: Location,
    /// Line 3: never truncated.
    pub validity: ValidityLabel,
    pub status: RowStatus,
}

/// Whether a row can be chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowStatus {
    Usable,
    Disabled(DisabledReason),
}

/// Why a visible row cannot sign (it sits in "Can't sign (n)").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisabledReason {
    Expired,
    NotYetValid,
    PinLocked,
    /// No algorithm in common with the request (`docs/ux.md` R6).
    Incompatible,
    /// Its token left while the window was open.
    Removed,
}

/// Why a certificate is not listed at all (counted in diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HiddenReason {
    Unparseable,
    NoPrivateKey,
    CertificateAuthority,
    /// KeyUsage without digitalSignature and nonRepudiation.
    KeyUsage,
    /// EKU present and only foreign purposes (serverAuth, codeSigning, …).
    ExtendedKeyUsage,
    /// Login certificate next to a signing sibling (`docs/ux.md` §5.8 rule 5).
    LoginSibling,
    /// Key type the app cannot sign with (EdDSA, DSA, unknown curve).
    UnsupportedKey,
}

/// The list as the window shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertList {
    /// Usable rows in display order.
    pub usable: Vec<CertRow>,
    /// "Can't sign (n)", collapsed by default.
    pub disabled: Vec<CertRow>,
    pub hidden: Vec<(Fingerprint, HiddenReason)>,
    /// Initial selection (`docs/ux.md` §5.9); `None` when nothing is usable.
    pub selected: Option<Fingerprint>,
}
