//! What the error notice says for each failure (`docs/ux.md` §15).

use websign_i18n::{Catalog, k};
use websign_ui_model::confirm::port::Failure;

use super::words;

/// Title, text and the "Technical details" line of an error notice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureText {
    pub title: String,
    pub body: String,
    /// Copyable detail (`CKR_DEVICE_ERROR (0x00000030)`), when there is one.
    pub technical: Option<String>,
}

pub fn failure_text(tr: &Catalog, failure: &Failure) -> FailureText {
    let (title, body, technical) = match failure {
        Failure::PinIncorrect { .. } => (k::PIN_INCORRECT, tr.tr(k::PIN_INCORRECT), None),
        Failure::PinLocked { tool, issuer } => {
            (k::PIN_LOCKED_TITLE, pin_locked(tr, tool, issuer), None)
        }
        Failure::TokenRemoved => (
            k::ERRORS_TOKEN_REMOVED_TITLE,
            tr.tr(k::ERRORS_TOKEN_REMOVED_BODY),
            None,
        ),
        Failure::DriverFailure { driver, native, .. } => (
            k::ERRORS_DRIVER_FAILURE_TITLE,
            tr.tr(k::ERRORS_DRIVER_FAILURE_BODY).arg("driver", driver),
            Some(native.clone()),
        ),
        Failure::UnsupportedAlgorithm { algorithm } => (
            k::ERRORS_UNSUPPORTED_ALGORITHM_TITLE,
            tr.tr(k::ERRORS_UNSUPPORTED_ALGORITHM_BODY)
                .arg("algorithm", words::algorithm(*algorithm)),
            None,
        ),
        Failure::CertificateUnavailable => (
            k::ERRORS_CERTIFICATE_UNAVAILABLE_TITLE,
            tr.tr(k::ERRORS_CERTIFICATE_UNAVAILABLE_BODY),
            None,
        ),
        Failure::Internal { detail } => (
            k::ERRORS_INTERNAL_TITLE,
            tr.tr(k::ERRORS_INTERNAL_BODY),
            Some(detail.clone()),
        ),
    };
    FailureText {
        title: tr.tr(title).to_string(),
        body: body.to_string(),
        technical: technical.filter(|detail| !detail.is_empty()),
    }
}

/// "Unlock it with the PUK in {tool} or contact {issuer}", or the generic
/// text when the vendor's tool is not known.
fn pin_locked<'a>(
    tr: &'a Catalog,
    tool: &Option<String>,
    issuer: &str,
) -> websign_i18n::Message<'a> {
    match tool {
        Some(tool) if !issuer.is_empty() => tr
            .tr(k::PIN_LOCKED_BODY)
            .arg("tool", tool)
            .arg("issuer", issuer),
        _ => tr.tr(k::PIN_LOCKED_BODY_GENERIC),
    }
}
