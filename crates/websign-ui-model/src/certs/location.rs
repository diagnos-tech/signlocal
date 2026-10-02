//! "Where the certificate is", in lay words (`docs/ux.md` §5.7).

use super::candidate::{CertCandidate, DeviceLabel, KeySource};

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
    let software = candidate.hardware == Some(false);
    let os_store = matches!(
        candidate.source,
        KeySource::Windows | KeySource::MacosKeychain
    );
    let place = match &candidate.device {
        _ if software && os_store => Place::Computer,
        Some(DeviceLabel::Token { name }) => Place::Token { name: name.clone() },
        Some(DeviceLabel::CardInReader { .. }) => Place::CardInReader,
        _ if software => Place::Computer,
        _ => Place::UnknownHardware,
    };
    Location {
        place,
        via_driver: matches!(candidate.source, KeySource::Driver { .. }),
    }
}
