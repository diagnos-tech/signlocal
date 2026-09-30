//! Stable error codes and the error message.
//!
//! The codes are the SDK's `WebSignError.code` values (`docs/ux.md` §15),
//! spelled identically on every transport. They are a public API: never
//! rename or reuse one; add new ones in a new protocol version.

use serde::{Deserialize, Serialize};

/// Why a request failed. Grouped by the party that detects it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum ErrorCode {
    // Detected by the SDK.
    /// The page never received the extension's announcement.
    ExtensionMissing,
    // Detected by the extension or the client library.
    /// Starting the app failed (not installed or not registered).
    AppMissing,
    /// The app is older than the caller requires (version or protocol).
    AppOutdated,
    /// The app requires a newer protocol than the extension speaks.
    ExtensionOutdated,
    /// A desktop client library speaks only protocols older than the app's.
    ClientOutdated,
    /// The page is not a secure context (`http:` other than localhost,
    /// `file:`, `data:`, another extension).
    InsecureOrigin,
    /// The caller withdrew the request (AbortSignal, `cancel`, `prepare` threw).
    Aborted,
    // Detected by the app.
    /// The person cancelled with nothing blocking visible.
    UserCancelled,
    /// Nobody decided within the time limit.
    Timeout,
    /// The window closed while the certificate list was empty.
    NoCertificates,
    /// The chosen or requested certificate is no longer reachable.
    CertificateUnavailable,
    /// The requested certificate is expired or not yet valid.
    CertificateNotValid,
    /// Malformed message, wrong digest length, unknown hash, broken sequence.
    InvalidRequest,
    /// The key or its driver cannot produce the requested algorithm.
    UnsupportedAlgorithm,
    /// Wrong PIN. Stays inside the window; only reported to callers of the
    /// CLI in unattended tests.
    PinIncorrect,
    /// The PIN is blocked.
    PinLocked,
    /// The token left while signing.
    TokenRemoved,
    /// The OS key store or the PKCS#11 driver failed for another reason.
    DriverFailure,
    /// Too many requests are already waiting.
    Busy,
    /// A bug in the app.
    Internal,
}

/// An error reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct WireError {
    pub code: ErrorCode,
    /// For developers, in English. Never shown to end users and never contains
    /// personal data (names, document numbers, digests, PINs).
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub details: Option<ErrorDetails>,
}

/// Machine-readable context for some codes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct ErrorDetails {
    /// `AppOutdated`/`ExtensionOutdated`/`ClientOutdated`: the version found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub installed: Option<String>,
    /// `AppOutdated`/`ExtensionOutdated`/`ClientOutdated`: the version needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub required: Option<String>,
    /// `DriverFailure`: the native status, e.g. `"CKR_DEVICE_ERROR (0x00000030)"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub native: Option<String>,
}

impl ErrorCode {
    /// Every code, in declaration order.
    pub const ALL: [ErrorCode; 20] = [
        ErrorCode::ExtensionMissing,
        ErrorCode::AppMissing,
        ErrorCode::AppOutdated,
        ErrorCode::ExtensionOutdated,
        ErrorCode::ClientOutdated,
        ErrorCode::InsecureOrigin,
        ErrorCode::Aborted,
        ErrorCode::UserCancelled,
        ErrorCode::Timeout,
        ErrorCode::NoCertificates,
        ErrorCode::CertificateUnavailable,
        ErrorCode::CertificateNotValid,
        ErrorCode::InvalidRequest,
        ErrorCode::UnsupportedAlgorithm,
        ErrorCode::PinIncorrect,
        ErrorCode::PinLocked,
        ErrorCode::TokenRemoved,
        ErrorCode::DriverFailure,
        ErrorCode::Busy,
        ErrorCode::Internal,
    ];

    /// The code as spelled on the wire (`"UserCancelled"`).
    pub const fn as_str(self) -> &'static str {
        match self {
            ErrorCode::ExtensionMissing => "ExtensionMissing",
            ErrorCode::AppMissing => "AppMissing",
            ErrorCode::AppOutdated => "AppOutdated",
            ErrorCode::ExtensionOutdated => "ExtensionOutdated",
            ErrorCode::ClientOutdated => "ClientOutdated",
            ErrorCode::InsecureOrigin => "InsecureOrigin",
            ErrorCode::Aborted => "Aborted",
            ErrorCode::UserCancelled => "UserCancelled",
            ErrorCode::Timeout => "Timeout",
            ErrorCode::NoCertificates => "NoCertificates",
            ErrorCode::CertificateUnavailable => "CertificateUnavailable",
            ErrorCode::CertificateNotValid => "CertificateNotValid",
            ErrorCode::InvalidRequest => "InvalidRequest",
            ErrorCode::UnsupportedAlgorithm => "UnsupportedAlgorithm",
            ErrorCode::PinIncorrect => "PinIncorrect",
            ErrorCode::PinLocked => "PinLocked",
            ErrorCode::TokenRemoved => "TokenRemoved",
            ErrorCode::DriverFailure => "DriverFailure",
            ErrorCode::Busy => "Busy",
            ErrorCode::Internal => "Internal",
        }
    }

    /// The process exit status `websign sign`/`choose` use for this code
    /// (`docs/architecture/desktop-api.md` §Exit codes).
    pub const fn exit_code(self) -> u8 {
        match self {
            ErrorCode::Internal => 1,
            ErrorCode::UserCancelled | ErrorCode::Aborted => 3,
            ErrorCode::Timeout => 4,
            ErrorCode::NoCertificates => 5,
            ErrorCode::CertificateUnavailable => 6,
            ErrorCode::CertificateNotValid => 7,
            ErrorCode::UnsupportedAlgorithm => 8,
            ErrorCode::PinLocked => 9,
            ErrorCode::PinIncorrect => 10,
            ErrorCode::TokenRemoved => 11,
            ErrorCode::DriverFailure => 12,
            ErrorCode::Busy => 13,
            ErrorCode::InvalidRequest | ErrorCode::InsecureOrigin => 14,
            ErrorCode::AppOutdated
            | ErrorCode::ExtensionOutdated
            | ErrorCode::ClientOutdated
            | ErrorCode::AppMissing
            | ErrorCode::ExtensionMissing => 15,
        }
    }
}
