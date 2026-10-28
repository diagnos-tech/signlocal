//! `OSStatus` results of Security.framework turned into [`KeystoreError`].
//!
//! The codes the app reacts to get their own variant; everything else keeps
//! the raw number plus its symbolic name, which is what people search for
//! when a report reaches them.

use security_framework::base::Error as SecError;

use crate::KeystoreError;

/// `errSecUserCanceled`: the user dismissed the OS PIN or password dialog.
const USER_CANCELED: i32 = -128;
/// `errSecAuthFailed`: wrong PIN, or wrong keychain password.
const AUTH_FAILED: i32 = -25293;
/// `errSecItemNotFound`: the key or identity is gone (deleted, token out).
const ITEM_NOT_FOUND: i32 = -25300;
/// `errSecInteractionNotAllowed`: the key needs a dialog the system may not
/// show now (a locked keychain in a session without UI).
const INTERACTION_NOT_ALLOWED: i32 = -25308;
/// `errSecUnimplemented`: the key does not implement the operation.
const UNIMPLEMENTED: i32 = -4;

/// Symbolic names of the other codes seen around key use.
const NAMES: &[(i32, &str)] = &[
    (-50, "errSecParam"),
    (-108, "errSecAllocate"),
    (-25291, "errSecNotAvailable"),
    (-25294, "errSecNoSuchKeychain"),
    (-25299, "errSecDuplicateItem"),
    (-26275, "errSecDecode"),
    (-34018, "errSecMissingEntitlement"),
];

/// Maps an `OSStatus` returned by the native call `api`.
pub fn from_status(api: &'static str, status: i32) -> KeystoreError {
    match status {
        USER_CANCELED => KeystoreError::Cancelled,
        AUTH_FAILED => KeystoreError::WrongPin,
        ITEM_NOT_FOUND => KeystoreError::NotFound,
        INTERACTION_NOT_ALLOWED => KeystoreError::PinRequired,
        UNIMPLEMENTED => KeystoreError::Unsupported(format!("{api}: errSecUnimplemented")),
        _ => KeystoreError::Native {
            api,
            code: i64::from(status),
            message: message(status),
        },
    }
}

/// `"<symbol>: <system text>"`, degrading to the bare number when neither is
/// known. The system text comes from `SecCopyErrorMessageString` and never
/// contains item data.
fn message(status: i32) -> String {
    let symbol = NAMES
        .iter()
        .find(|(code, _)| *code == status)
        .map(|(_, name)| *name);
    let text = SecError::from_code(status).message();
    match (symbol, text) {
        (Some(symbol), Some(text)) => format!("{symbol}: {text}"),
        (Some(symbol), None) => symbol.to_owned(),
        (None, Some(text)) => text,
        (None, None) => format!("OSStatus {status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_the_app_reacts_to_have_their_own_variant() {
        assert!(matches!(from_status("x", -128), KeystoreError::Cancelled));
        assert!(matches!(from_status("x", -25293), KeystoreError::WrongPin));
        assert!(matches!(from_status("x", -25300), KeystoreError::NotFound));
        assert!(matches!(
            from_status("x", -25308),
            KeystoreError::PinRequired
        ));
        assert!(matches!(
            from_status("x", -4),
            KeystoreError::Unsupported(_)
        ));
    }

    #[test]
    fn other_codes_keep_api_number_and_symbol() {
        let KeystoreError::Native { api, code, message } =
            from_status("SecKeyCreateSignature", -50)
        else {
            panic!("expected Native");
        };
        assert_eq!(api, "SecKeyCreateSignature");
        assert_eq!(code, -50);
        assert!(message.starts_with("errSecParam"), "{message}");
    }

    #[test]
    fn unknown_codes_still_say_something() {
        let KeystoreError::Native { message, .. } = from_status("x", 123_456) else {
            panic!("expected Native");
        };
        assert!(!message.is_empty());
    }
}
