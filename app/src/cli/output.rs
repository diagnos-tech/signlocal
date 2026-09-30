//! How commands answer: one JSON document on stdout for machines, a short
//! line on stderr for people, and the stable exit code of
//! `docs/architecture/desktop-api.md` §4.

use std::io::Write;
use std::process::ExitCode;

use websign_i18n::{Catalog, Key, k};
use websign_protocol::{AppMessage, ErrorCode, WireError};

/// Exit status for success.
pub const SUCCESS: u8 = 0;
/// Exit status for a failure that has no protocol code (`Internal`).
pub const FAILURE: u8 = 1;

/// Writes `json` and a newline to stdout in one call. A closed stdout (the
/// reader went away) is not worth failing over: the exit code still tells.
pub fn print_json(json: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{json}");
    let _ = out.flush();
}

/// The final protocol message of `sign`/`choose`, as the documented JSON
/// object (no envelope).
pub fn message_json(message: &AppMessage) -> String {
    serde_json::to_string(message)
        .unwrap_or_else(|_| internal_json("the reply could not be encoded"))
}

/// `{"type":"error","code":"Internal","message":…}` without going through
/// serde, for the one case where serialization itself failed.
fn internal_json(message: &str) -> String {
    format!(r#"{{"type":"error","code":"Internal","message":"{message}"}}"#)
}

/// An error message for `code`.
pub fn error(code: ErrorCode, message: impl Into<String>) -> AppMessage {
    AppMessage::Error(WireError {
        code,
        message: message.into(),
        details: None,
    })
}

/// Prints `message` as JSON, tells the person on stderr when it is an
/// error, and returns its exit code.
pub fn finish(message: &AppMessage, catalog: &Catalog) -> ExitCode {
    print_json(&message_json(message));
    match message {
        AppMessage::Error(error) => {
            eprintln!("websign: {}", person_text(error, catalog));
            ExitCode::from(error.code.exit_code())
        }
        _ => ExitCode::from(SUCCESS),
    }
}

/// The window's title for `code` in the person's language, or the English
/// developer message when the catalog has none for it.
pub fn person_text(error: &WireError, catalog: &Catalog) -> String {
    match title_key(error.code) {
        Some(key) => catalog.tr(key).to_string(),
        None => error.message.clone(),
    }
}

/// The `[site.errors.*]` title a person reads for `code`.
fn title_key(code: ErrorCode) -> Option<Key> {
    Some(match code {
        ErrorCode::AppMissing => k::SITE_ERRORS_APP_MISSING_TITLE,
        ErrorCode::AppOutdated => k::SITE_ERRORS_APP_OUTDATED_TITLE,
        ErrorCode::InsecureOrigin => k::SITE_ERRORS_INSECURE_ORIGIN_TITLE,
        ErrorCode::Aborted => k::SITE_ERRORS_ABORTED_TITLE,
        ErrorCode::UserCancelled => k::SITE_ERRORS_USER_CANCELLED_TITLE,
        ErrorCode::Timeout => k::SITE_ERRORS_TIMEOUT_TITLE,
        ErrorCode::NoCertificates => k::SITE_ERRORS_NO_CERTIFICATES_TITLE,
        ErrorCode::CertificateUnavailable => k::SITE_ERRORS_CERTIFICATE_UNAVAILABLE_TITLE,
        ErrorCode::CertificateNotValid => k::SITE_ERRORS_CERTIFICATE_NOT_VALID_TITLE,
        ErrorCode::UnsupportedAlgorithm => k::SITE_ERRORS_UNSUPPORTED_ALGORITHM_TITLE,
        ErrorCode::PinLocked => k::SITE_ERRORS_PIN_LOCKED_TITLE,
        ErrorCode::TokenRemoved => k::SITE_ERRORS_TOKEN_REMOVED_TITLE,
        ErrorCode::DriverFailure => k::SITE_ERRORS_DRIVER_FAILURE_TITLE,
        ErrorCode::Busy => k::SITE_ERRORS_BUSY_TITLE,
        ErrorCode::Internal => k::SITE_ERRORS_INTERNAL_TITLE,
        ErrorCode::ExtensionMissing
        | ErrorCode::ExtensionOutdated
        | ErrorCode::ClientOutdated
        | ErrorCode::InvalidRequest
        | ErrorCode::PinIncorrect => return None,
    })
}
