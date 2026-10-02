//! Security.framework and CryptoTokenKit `CFError`s turned into
//! [`KeystoreError`], so the app can tell a dismissed PIN dialog, a wrong or
//! blocked PIN and a pulled token apart from a real failure.

use core_foundation::error::CFError;

use super::{status, user_info};
use crate::KeystoreError;

/// `NSOSStatusErrorDomain`: the error code is an `OSStatus`.
const OSSTATUS_DOMAIN: &str = "NSOSStatusErrorDomain";
/// The value of `TKErrorDomain`. Security.framework passes token failures on
/// in this domain unchanged (see `SecCTKKey.m`), so a smart card PIN dialog
/// that is cancelled arrives as a CryptoTokenKit code, not an `OSStatus`.
const CTK_DOMAIN: &str = "CryptoTokenKit";

/// `TKErrorCode` values (`TKError.h`), with their symbolic names.
mod tk {
    pub const NOT_IMPLEMENTED: isize = -1;
    pub const CANCELED_BY_USER: isize = -4;
    pub const AUTHENTICATION_FAILED: isize = -5;
    pub const OBJECT_NOT_FOUND: isize = -6;
    pub const TOKEN_NOT_FOUND: isize = -7;
    pub const AUTHENTICATION_NEEDED: isize = -9;

    pub const NAMES: &[(isize, &str)] = &[
        (NOT_IMPLEMENTED, "TKErrorCodeNotImplemented"),
        (-2, "TKErrorCodeCommunicationError"),
        (-3, "TKErrorCodeCorruptedData"),
        (CANCELED_BY_USER, "TKErrorCodeCanceledByUser"),
        (AUTHENTICATION_FAILED, "TKErrorCodeAuthenticationFailed"),
        (OBJECT_NOT_FOUND, "TKErrorCodeObjectNotFound"),
        (TOKEN_NOT_FOUND, "TKErrorCodeTokenNotFound"),
        (-8, "TKErrorCodeBadParameter"),
        (AUTHENTICATION_NEEDED, "TKErrorCodeAuthenticationNeeded"),
    ];
}

/// What [`classify`] may need beyond domain and code; each is read only for
/// the errors that need it.
struct Details<D, R> {
    /// The system's description of the error.
    describe: D,
    /// PIN attempts left, when the token driver reported them.
    remaining_attempts: R,
}

/// Maps the `CFError` reported by the native call `api`.
pub fn from_cf_error(api: &'static str, error: &CFError) -> KeystoreError {
    let domain = error.domain().to_string();
    let details = Details {
        describe: || error.description().to_string(),
        remaining_attempts: || user_info::remaining_attempts(error),
    };
    classify(api, &domain, error.code(), details)
}

/// Pure core of [`from_cf_error`].
fn classify(
    api: &'static str,
    domain: &str,
    code: isize,
    details: Details<impl FnOnce() -> String, impl FnOnce() -> Option<i64>>,
) -> KeystoreError {
    match domain {
        OSSTATUS_DOMAIN => match i32::try_from(code) {
            Ok(code) => status::from_status(api, code),
            Err(_) => native(api, domain, code, &(details.describe)()),
        },
        CTK_DOMAIN => match code {
            tk::CANCELED_BY_USER => KeystoreError::Cancelled,
            // Hardware check (docs/compatibility.md): which userInfo entry
            // carries the attempts left is unconfirmed on real tokens, so a
            // blocked PIN may still be reported as WrongPin.
            tk::AUTHENTICATION_FAILED if (details.remaining_attempts)() == Some(0) => {
                KeystoreError::PinLocked
            }
            tk::AUTHENTICATION_FAILED => KeystoreError::WrongPin,
            tk::AUTHENTICATION_NEEDED => KeystoreError::PinRequired,
            tk::TOKEN_NOT_FOUND => KeystoreError::TokenRemoved,
            tk::OBJECT_NOT_FOUND => KeystoreError::NotFound,
            tk::NOT_IMPLEMENTED => {
                KeystoreError::Unsupported(format!("{api}: TKErrorCodeNotImplemented"))
            }
            _ => native(api, domain, code, &(details.describe)()),
        },
        _ => native(api, domain, code, &(details.describe)()),
    }
}

fn native(api: &'static str, domain: &str, code: isize, description: &str) -> KeystoreError {
    let symbol = match domain {
        CTK_DOMAIN => tk::NAMES.iter().find(|(c, _)| *c == code).map(|(_, n)| *n),
        _ => None,
    };
    KeystoreError::Native {
        api,
        // CFIndex is 64 bits on every macOS target.
        code: code as i64,
        message: match symbol {
            Some(symbol) => format!("{domain} {symbol}: {description}"),
            None => format!("{domain}: {description}"),
        },
    }
}

#[cfg(test)]
mod tests;
