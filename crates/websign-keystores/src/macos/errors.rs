//! Security.framework and CryptoTokenKit failures turned into
//! [`KeystoreError`], so the app can tell a dismissed PIN dialog or a wrong
//! PIN apart from a real failure.

use core_foundation::error::CFError;
use security_framework::base::Error as SecError;

use crate::KeystoreError;

/// `errSecUserCanceled`: the user dismissed the OS PIN or password dialog.
const ERR_SEC_USER_CANCELED: i32 = -128;
/// `errSecAuthFailed`: wrong PIN, or wrong keychain password.
const ERR_SEC_AUTH_FAILED: i32 = -25293;

/// `NSOSStatusErrorDomain`: the error code is an `OSStatus`.
const OSSTATUS_DOMAIN: &str = "NSOSStatusErrorDomain";
/// The value of `TKErrorDomain`. Security.framework passes token failures on
/// in this domain unchanged (see `SecCTKKey.m`), so a smart card PIN dialog
/// that is cancelled arrives as a CryptoTokenKit code, not an `OSStatus`.
const CTK_DOMAIN: &str = "CryptoTokenKit";
/// `TKErrorCodeCanceledByUser` (`TKError.h`).
const CTK_CANCELED_BY_USER: isize = -4;
/// `TKErrorCodeAuthenticationFailed` (`TKError.h`).
const CTK_AUTHENTICATION_FAILED: isize = -5;

/// Maps an `OSStatus` returned by the native call `api`.
pub fn from_status(api: &'static str, status: i32) -> KeystoreError {
    match status {
        ERR_SEC_USER_CANCELED => KeystoreError::Cancelled,
        ERR_SEC_AUTH_FAILED => KeystoreError::WrongPin,
        _ => KeystoreError::Native {
            api,
            code: i64::from(status),
            message: status_message(status),
        },
    }
}

/// Maps the `CFError` reported by the native call `api`.
pub fn from_cf_error(api: &'static str, error: &CFError) -> KeystoreError {
    let domain = error.domain().to_string();
    let code = error.code();
    classify(api, &domain, code, || error.description().to_string())
}

/// Pure core of [`from_cf_error`]; `describe` is only called for errors that
/// are reported as they are.
fn classify(
    api: &'static str,
    domain: &str,
    code: isize,
    describe: impl FnOnce() -> String,
) -> KeystoreError {
    match domain {
        OSSTATUS_DOMAIN => match i32::try_from(code) {
            Ok(status) => from_status(api, status),
            Err(_) => native(api, domain, code, &describe()),
        },
        CTK_DOMAIN if code == CTK_CANCELED_BY_USER => KeystoreError::Cancelled,
        CTK_DOMAIN if code == CTK_AUTHENTICATION_FAILED => KeystoreError::WrongPin,
        // TODO(gustavo): a blocked PIN arrives as AuthenticationFailed with
        // zero tries left in the userInfo; map it to PinLocked once seen on a
        // real token.
        _ => native(api, domain, code, &describe()),
    }
}

fn native(api: &'static str, domain: &str, code: isize, description: &str) -> KeystoreError {
    KeystoreError::Native {
        api,
        // CFIndex is 64 bits on every macOS target.
        code: code as i64,
        message: format!("{domain}: {description}"),
    }
}

/// `SecCopyErrorMessageString`, with the bare number when the system has no text.
fn status_message(status: i32) -> String {
    SecError::from_code(status)
        .message()
        .unwrap_or_else(|| format!("OSStatus {status}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unreachable_description() -> String {
        panic!("description is only needed for errors reported as they are")
    }

    #[test]
    fn cancellation_in_both_domains() {
        for (domain, code) in [(OSSTATUS_DOMAIN, -128), (CTK_DOMAIN, -4)] {
            let error = classify("x", domain, code, unreachable_description);
            assert!(matches!(error, KeystoreError::Cancelled), "{domain} {code}");
        }
    }

    #[test]
    fn wrong_pin_in_both_domains() {
        for (domain, code) in [(OSSTATUS_DOMAIN, -25293), (CTK_DOMAIN, -5)] {
            let error = classify("x", domain, code, unreachable_description);
            assert!(matches!(error, KeystoreError::WrongPin), "{domain} {code}");
        }
    }

    #[test]
    fn anything_else_keeps_api_code_and_domain() {
        let error = classify("SecKeyCreateSignature", CTK_DOMAIN, -7, || {
            "token not found".to_owned()
        });
        let KeystoreError::Native { api, code, message } = error else {
            panic!("expected Native, got {error:?}");
        };
        assert_eq!(api, "SecKeyCreateSignature");
        assert_eq!(code, -7);
        assert_eq!(message, "CryptoTokenKit: token not found");
    }

    #[test]
    fn osstatus_messages_come_from_the_system() {
        // errSecItemNotFound
        let KeystoreError::Native { code, message, .. } = from_status("x", -25300) else {
            panic!("expected Native");
        };
        assert_eq!(code, -25300);
        assert!(!message.is_empty());
    }
}
