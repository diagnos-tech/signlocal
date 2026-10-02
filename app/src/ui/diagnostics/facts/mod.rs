//! What the diagnostics window knows about this computer: browsers, devices,
//! token drivers and certificates, gathered in one pass off the UI thread
//! ([`collect()`]) and then only read.
//!
//! Facts are plain data so the window, the traffic lights and the report can
//! be tested with hand-built values. Anything that could identify a person
//! stays out of the parts the report reads: paths have the user folder
//! replaced ([`home`]), ATRs are masked when shown, and certificate details
//! only live in [`CertificatesFact::list`] for the Certificates tab.

pub mod browsers;
mod certificates;
mod collect;
mod devices;
mod drivers;
mod hidden_rows;
pub mod home;
mod system;

use std::collections::HashMap;

use websign_core::Fingerprint;
use websign_host::store::{ConnectionRecord, ErrorRecord};
use websign_registration::Browser;
use websign_registration::detect::BrowserPackaging;
use websign_registration::status::RegistrationState;
use websign_ui_model::certs::{CertList, CertRow};

#[cfg(test)]
pub use certificates::from_candidates;
pub use collect::{ScanInput, collect};
pub use hidden_rows::ShownReason;
pub use system::packaging_name;

/// Everything one scan found.
#[derive(Debug, Clone)]
pub struct Facts {
    pub browsers: Vec<BrowserFact>,
    pub devices: DevicesFact,
    pub drivers: Vec<DriverFact>,
    pub certificates: CertificatesFact,
    /// Oldest first, as the error store keeps them.
    pub recent_errors: Vec<ErrorRecord>,
    /// `"Ubuntu 24.04.1 LTS (6.8.0-45-generic)"`.
    pub os: String,
}

/// One installed browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserFact {
    pub browser: Browser,
    pub version: Option<String>,
    pub packaging: BrowserPackaging,
    pub registration: RegistrationState,
    /// The extension's last `hello` from this browser, if any.
    pub connection: Option<ConnectionRecord>,
}

/// Tokens, cards and readers, and the Linux card service.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DevicesFact {
    /// Linux only: whether `pcscd` answers.
    pub pcscd_running: Option<bool>,
    pub tokens: Vec<TokenFact>,
    pub readers: Vec<ReaderFact>,
}

/// A USB token (or an unknown smart card device).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenFact {
    /// `"0529:0620"`.
    pub vid_pid: String,
    /// `devices.json` id and name, when known.
    pub hint: Option<HintFact>,
    pub certificates: CertPresence,
}

/// A PC/SC reader and the card in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReaderFact {
    /// Without serial numbers (`anonymous_reader_name`).
    pub name: String,
    pub card: Option<CardFact>,
}

/// The card in a reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardFact {
    /// Uppercase hex, unmasked: mask it before showing or reporting.
    pub atr: Option<String>,
    pub hint: Option<HintFact>,
    pub certificates: CertPresence,
}

/// The `devices.json` entry a device matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintFact {
    pub id: String,
    pub name: String,
    /// Vendor middleware name, and its download page for this OS.
    pub driver: Option<String>,
    pub download: Option<String>,
}

/// Whether a device brought certificates to the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertPresence {
    /// Linked to this many listed certificates.
    Found(u32),
    /// None, and the device is known well enough to say so (§6.1).
    Missing,
    /// None linked, but a hardware key with no known device exists: it may
    /// live here ("We can't tell if it has certificates").
    Unknown,
}

impl CertPresence {
    /// The count the report prints.
    pub fn count(self) -> u32 {
        match self {
            CertPresence::Found(count) => count,
            CertPresence::Missing | CertPresence::Unknown => 0,
        }
    }
}

/// A PKCS#11 module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverFact {
    /// File name (`opensc-pkcs11.so`).
    pub name: String,
    /// Full path with the user folder replaced; the file name alone when the
    /// key store did not say where it found the module.
    pub path: String,
    /// `Ok(tokens with certificates)` or the failure reason.
    pub result: Result<u32, String>,
    /// Added with "Add driver…" (removable).
    pub user_added: bool,
    /// The exact path kept in settings, to remove it again.
    pub setting: Option<std::path::PathBuf>,
}

/// The certificate list as the Certificates tab shows it, plus what the
/// report counts.
#[derive(Debug, Clone)]
pub struct CertificatesFact {
    pub list: CertList,
    /// Left out of the list but shown under "Can't sign" with their reason.
    pub hidden_rows: Vec<(CertRow, ShownReason)>,
    /// DER per fingerprint, for "Details" (the OS viewer).
    pub der: HashMap<Fingerprint, Vec<u8>>,
    /// Extra paths to an already listed key (`docs/ux.md` §5.11).
    pub deduplicated: u32,
}
