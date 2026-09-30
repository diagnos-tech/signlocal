//! Final error replies and how the window is told about them.

use websign_protocol::{AppMessage, ErrorCode, WireError};
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::{Finish, RequestKey};

use super::Effect;

/// The reply that ends a request with `code`. Messages are English and for
/// developers: they name the condition, never the person or the site.
pub(super) fn error_reply(code: ErrorCode) -> Effect {
    Effect::Send(AppMessage::Error(WireError {
        code,
        message: describe(code).to_owned(),
        details: None,
    }))
}

/// The window's closing command for a request that ended with `code`: a
/// timeout says so; every other end just moves the window on.
pub(super) fn finished(key: RequestKey, code: ErrorCode) -> Effect {
    let finish = match code {
        ErrorCode::Timeout => Finish::Timeout,
        _ => Finish::Aborted,
    };
    Effect::Ui(UiCommand::Finished { key, finish })
}

fn describe(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::ExtensionMissing => "the extension is not installed",
        ErrorCode::AppMissing => "the app is not installed",
        ErrorCode::AppOutdated => "the app is older than the caller requires",
        ErrorCode::ExtensionOutdated => "the extension is older than the app requires",
        ErrorCode::ClientOutdated => "the client is older than the app requires",
        ErrorCode::InsecureOrigin => "the origin is not a secure context",
        ErrorCode::Aborted => "the caller cancelled the request",
        ErrorCode::UserCancelled => "the person cancelled",
        ErrorCode::Timeout => "nobody answered in time",
        ErrorCode::NoCertificates => "no certificates were found",
        ErrorCode::CertificateUnavailable => "the certificate is no longer available",
        ErrorCode::CertificateNotValid => "the certificate is not valid now",
        ErrorCode::InvalidRequest => "the request is not valid",
        ErrorCode::UnsupportedAlgorithm => "the key cannot produce the requested algorithm",
        ErrorCode::PinIncorrect => "the PIN is incorrect",
        ErrorCode::PinLocked => "the PIN is locked",
        ErrorCode::TokenRemoved => "the token was removed",
        ErrorCode::DriverFailure => "the key store failed",
        ErrorCode::Busy => "too many requests are waiting",
        ErrorCode::Internal => "internal error",
    }
}
