//! A description of the confirmation window at one instant: everything the
//! egui renderer needs, nothing it must decide. Text is chosen by the
//! renderer from these values (message keys live in the app).

use websign_core::Fingerprint;
use websign_protocol::types::HashName;
use websign_protocol::{ErrorCode, VerificationCode};

use super::machine::ConfirmState;
use super::port::{CallerView, Failure, Mode};
use crate::certs::CertList;
use crate::possible::PossibleCard;

/// The whole window.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmView {
    pub state: ConfirmState,
    pub mode: Mode,
    pub header: HeaderView,
    pub code: CodeCard,
    pub list: Option<CertList>,
    pub selected: Option<Fingerprint>,
    pub possible: Vec<PossibleCard>,
    pub pin: PinBlock,
    pub remember: RememberBox,
    pub banner: Option<Failure>,
    pub footer: FooterView,
}

/// Header: eyebrow, caller, alerts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderView {
    pub caller: CallerView,
    pub remembered: bool,
    /// `Some((current, total))` when more than one request waits.
    pub queue: Option<(u32, u32)>,
}

/// The verification-code card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeCard {
    /// No selection (empty list): the card is not shown.
    Hidden,
    /// Waiting for Continue (D11): "Choose the certificate and click Continue…".
    ContinueHint,
    /// Waiting for the digest; `skeleton` once 150 ms have passed.
    Preparing { skeleton: bool },
    Ready {
        code: VerificationCode,
        hash: HashName,
    },
    /// Choose mode: "The site will receive name, type, issuer and validity…".
    SelectShares,
}

/// The PIN area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinBlock {
    Hidden,
    /// Our field. `valid` = the typed length is inside `length`.
    Field {
        card: bool,
        length: Option<(u32, u32)>,
        valid: bool,
        error: Option<PinError>,
    },
    /// "Windows/macOS will ask for your PIN in its own window."
    OsPrompt {
        now: bool,
    },
    /// "Enter your PIN on the reader's keypad."
    PinPad {
        now: bool,
    },
    /// "Token unlocked for this session".
    Unlocked,
    /// The locked warning replaces the field.
    Locked,
}

/// The PIN field's error line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinError {
    Incorrect,
    IncorrectLow,
    IncorrectFinal,
}

/// "Remember this site" checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RememberBox {
    /// Remembered callers and choose-less flows.
    Hidden,
    Enabled {
        checked: bool,
    },
    /// IP or IDN origin: "can't be remembered".
    Disabled,
}

/// Footer: hint, buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterView {
    pub hint: FooterHint,
    pub primary: PrimaryButton,
    pub primary_enabled: bool,
    /// False only while the OS or the PIN pad is signing (no API can abort).
    pub cancel_enabled: bool,
    /// Windows puts the primary button first.
    pub primary_first: bool,
}

/// The footer's left text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterHint {
    None,
    /// "This request expires in {seconds} s" (last 30 s).
    ExpiresIn {
        seconds: u32,
    },
    OsPinPrompt,
    OpenDiagnostics,
}

/// The primary button's label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryButton {
    Sign,
    Signing,
    Retry,
    Continue,
    UseCertificate,
}

/// Why the error banner is shown, when it maps to a code.
pub fn banner_code(failure: &Failure) -> ErrorCode {
    let _ = failure;
    todo!("SPEC.md §2.4")
}
