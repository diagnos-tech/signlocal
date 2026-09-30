//! "Where the certificate is", in lay words (`docs/ux.md` §5.7).

use super::candidate::CertCandidate;

/// The location phrase of line 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub place: Place,
    /// Appends " · via driver" (the key goes through a PKCS#11 driver, which
    /// explains why our PIN field appears).
    pub via_driver: bool,
}

/// The place itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// Software key in the OS store: "On this computer".
    Computer,
    /// "Token SafeNet eToken 5110".
    Token { name: String },
    /// "Card in reader".
    CardInReader,
    /// "Token or card".
    UnknownHardware,
}

/// The location of `candidate`.
pub fn location(candidate: &CertCandidate) -> Location {
    let _ = candidate;
    todo!("SPEC.md §1.4")
}
