//! The contract between the host engine and the confirmation window.
//!
//! The host sends [`UiCommand`]s; the window answers with [`UiEvent`]s. Both
//! travel over channels between the engine thread and the UI thread. The
//! window never sees digests, only verification codes; the host never sees
//! the screen, only decisions.

use secrecy::SecretString;
use websign_core::Fingerprint;
use websign_core::present::caller::CallerLabel;
use websign_core::present::origin::FormattedOrigin;
use websign_protocol::ErrorCode;
use websign_protocol::VerificationCode;
use websign_protocol::messages::DiagnosticsTab;
use websign_protocol::types::{BrowserName, HashName, SignatureAlgorithmName};

use crate::certs::{CertCandidate, ListContext};
use crate::possible::PossibleCard;

/// Identifies one request across host and window (not the wire id, which is
/// only unique per connection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequestKey(pub u64);

/// What the request wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Sign {
        hash: HashName,
    },
    /// `certificates()` for a caller that is not remembered.
    Choose,
}

/// Who asks, as far as the app itself knows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerView {
    Web {
        origin: FormattedOrigin,
        /// The tab's top-level origin when the request comes from a frame of
        /// another origin ("Inside a page from …").
        top: Option<FormattedOrigin>,
        browser: BrowserName,
    },
    Desktop {
        label: CallerLabel,
    },
}

/// A request reaching the front of the queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRequest {
    pub key: RequestKey,
    pub mode: Mode,
    pub caller: CallerView,
    /// The caller has remembered consent: its certificate is released without
    /// "Continue" (`docs/plan.md` D11) and the chip says "Allowed site".
    pub remembered: bool,
    /// "Remember this site" may be offered (false for IP and IDN origins).
    pub can_remember: bool,
    /// 1-based position and queue length, for "Signature request 1 of 3".
    pub position: (u32, u32),
    /// Seconds until `Timeout`; the footer counts down in the last 30.
    pub timeout_secs: u32,
}

/// Why signing failed, in the terms the window needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    PinIncorrect {
        count_low: bool,
        final_try: bool,
    },
    PinLocked {
        /// `devices.json` `pinUnlock.tool`, when known.
        tool: Option<String>,
        /// The issuing authority's short name.
        issuer: String,
    },
    TokenRemoved,
    DriverFailure {
        /// Key store or driver name for the message.
        driver: String,
        /// Technical detail, e.g. `"CKR_DEVICE_ERROR (0x00000030)"`.
        native: String,
        /// Another path to the same key exists ("Try through the token driver").
        alternate: bool,
    },
    UnsupportedAlgorithm {
        algorithm: SignatureAlgorithmName,
    },
    CertificateUnavailable,
    Internal {
        detail: String,
    },
}

/// How a request ended, for the closing animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finish {
    Signed,
    Chosen,
    /// The tab closed or navigated.
    SiteCancelled,
    Timeout,
    /// The caller withdrew the request (no text; the window just moves on).
    Aborted,
}

/// Host → window.
#[derive(Debug, Clone, PartialEq)]
pub enum UiCommand {
    /// Show this request (first open, or the next in the queue).
    Open(OpenRequest),
    /// The current certificate candidates (full replacement; the window
    /// merges with [`crate::certs::CertList::append`]).
    Certificates {
        key: RequestKey,
        candidates: Vec<CertCandidate>,
        possible: Vec<PossibleCard>,
        context: ListContext,
    },
    /// Listing is taking long; `device` names what is being read.
    SlowListing {
        key: RequestKey,
        device: Option<String>,
    },
    /// The digest for certificate `fingerprint` was asked for.
    DigestPending {
        key: RequestKey,
        fingerprint: Fingerprint,
    },
    /// The digest arrived; show its code.
    DigestReady {
        key: RequestKey,
        fingerprint: Fingerprint,
        code: VerificationCode,
    },
    /// Signing started; the OS may be showing its PIN dialog.
    Signing {
        key: RequestKey,
    },
    Failed {
        key: RequestKey,
        failure: Failure,
    },
    Finished {
        key: RequestKey,
        finish: Finish,
    },
    /// Queue length changed while `key` is on screen.
    Queue {
        key: RequestKey,
        position: (u32, u32),
    },
    /// Nothing left to show: hide the window (the process may live on).
    Hide,
}

/// Window → host.
#[derive(Debug)]
pub enum UiEvent {
    /// The person moved the selection. For a remembered caller the host asks
    /// for a new digest; otherwise it waits for `Continue`.
    Selected {
        key: RequestKey,
        fingerprint: Fingerprint,
    },
    /// Release the certificate to a caller that is not remembered (D11).
    Continue {
        key: RequestKey,
        fingerprint: Fingerprint,
    },
    /// Sign with `fingerprint` through path `via` (0 = primary).
    Sign {
        key: RequestKey,
        fingerprint: Fingerprint,
        via: usize,
        /// Only for [`crate::certs::PinMode::App`]; zeroized on drop.
        pin: Option<SecretString>,
        remember: bool,
    },
    /// Choose mode: send `fingerprint` to the caller.
    Choose {
        key: RequestKey,
        fingerprint: Fingerprint,
        remember: bool,
    },
    /// Cancel, Esc or the close button. `code` follows `docs/ux.md` §15
    /// ([`super::cancel::cancel_code`]).
    Cancel { key: RequestKey, code: ErrorCode },
    /// "Scan again" / "I installed it, scan again".
    Rescan { key: RequestKey },
    /// "Open diagnostics": the host starts a separate process.
    OpenDiagnostics { tab: Option<DiagnosticsTab> },
    /// "View in system" (`docs/ux.md` §5.12): the host opens the OS
    /// certificate viewer with the certificate's DER, which the window never
    /// holds, owned by the confirmation window.
    ViewCertificate {
        key: RequestKey,
        fingerprint: Fingerprint,
    },
}
