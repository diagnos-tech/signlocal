//! Windows status codes turned into [`KeystoreError`], so the app can tell a
//! cancelled PIN dialog, a wrong PIN or a pulled card apart from a real
//! failure.

use windows::Win32::Foundation::{
    CRYPT_E_NO_KEY_PROPERTY, ERROR_CANCELLED, NTE_BAD_ALGID, NTE_NOT_SUPPORTED, NTE_SILENT_CONTEXT,
    NTE_USER_CANCELLED, SCARD_E_INVALID_CHV, SCARD_E_NO_READERS_AVAILABLE, SCARD_E_NO_SMARTCARD,
    SCARD_E_READER_UNAVAILABLE, SCARD_E_UNSUPPORTED_FEATURE, SCARD_W_CANCELLED_BY_USER,
    SCARD_W_CARD_NOT_AUTHENTICATED, SCARD_W_CHV_BLOCKED, SCARD_W_REMOVED_CARD,
    SCARD_W_UNPOWERED_CARD, SCARD_W_WRONG_CHV,
};
use windows::core::HRESULT;

use super::status_names::symbolic_name;
use crate::KeystoreError;

/// The same outcome is spelled differently by each layer: smart card
/// minidrivers report `SCARD_*`, CNG providers `NTE_*`, and some CSPs plain
/// Win32 errors.
const CANCELLED: [HRESULT; 3] = [
    SCARD_W_CANCELLED_BY_USER,
    NTE_USER_CANCELLED,
    HRESULT::from_win32(ERROR_CANCELLED.0),
];
const WRONG_PIN: [HRESULT; 2] = [SCARD_W_WRONG_CHV, SCARD_E_INVALID_CHV];
/// `NTE_SILENT_CONTEXT`: the key needs UI (its PIN dialog) and the run is
/// silent; `SCARD_W_CARD_NOT_AUTHENTICATED`: the card wants a PIN first.
const PIN_REQUIRED: [HRESULT; 2] = [NTE_SILENT_CONTEXT, SCARD_W_CARD_NOT_AUTHENTICATED];
const CARD_GONE: [HRESULT; 5] = [
    SCARD_W_REMOVED_CARD,
    SCARD_W_UNPOWERED_CARD,
    SCARD_E_NO_SMARTCARD,
    SCARD_E_READER_UNAVAILABLE,
    SCARD_E_NO_READERS_AVAILABLE,
];
const UNSUPPORTED: [HRESULT; 3] = [
    NTE_BAD_ALGID,
    NTE_NOT_SUPPORTED,
    SCARD_E_UNSUPPORTED_FEATURE,
];

/// Maps the error of the native call `api`.
pub fn native(api: &'static str, error: &windows::core::Error) -> KeystoreError {
    from_code(api, error.code())
}

/// Maps a status code; anything without a meaning of its own keeps the call,
/// the code and its symbolic name for the diagnostics report.
pub fn from_code(api: &'static str, code: HRESULT) -> KeystoreError {
    if CANCELLED.contains(&code) {
        KeystoreError::Cancelled
    } else if WRONG_PIN.contains(&code) {
        KeystoreError::WrongPin
    } else if code == SCARD_W_CHV_BLOCKED {
        KeystoreError::PinLocked
    } else if PIN_REQUIRED.contains(&code) {
        KeystoreError::PinRequired
    } else if CARD_GONE.contains(&code) {
        KeystoreError::TokenRemoved
    } else if code == CRYPT_E_NO_KEY_PROPERTY {
        KeystoreError::NotFound
    } else if UNSUPPORTED.contains(&code) {
        KeystoreError::Unsupported(format!("{api}: {}", describe(code)))
    } else {
        KeystoreError::Native {
            api,
            // Unsigned, so it prints as the familiar 0x8010006b.
            code: i64::from(code.0 as u32),
            message: describe(code),
        }
    }
}

/// `NTE_BAD_KEYSET: Keyset does not exist.`, or only the system message
/// for codes without a known name.
fn describe(code: HRESULT) -> String {
    let message = code.message();
    match symbolic_name(code) {
        Some(name) if message.is_empty() => name.to_owned(),
        Some(name) => format!("{name}: {message}"),
        None => message,
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::{
        ERROR_CANCELLED, NTE_BAD_ALGID, NTE_BAD_KEYSET, NTE_SILENT_CONTEXT, NTE_USER_CANCELLED,
        SCARD_E_INVALID_CHV, SCARD_E_NO_SMARTCARD, SCARD_W_CANCELLED_BY_USER, SCARD_W_CHV_BLOCKED,
        SCARD_W_REMOVED_CARD, SCARD_W_WRONG_CHV,
    };
    use windows::core::HRESULT;

    use super::from_code;
    use crate::KeystoreError;

    #[test]
    fn user_cancellation_in_every_spelling() {
        for code in [
            SCARD_W_CANCELLED_BY_USER,
            NTE_USER_CANCELLED,
            HRESULT::from_win32(ERROR_CANCELLED.0),
        ] {
            assert!(matches!(from_code("x", code), KeystoreError::Cancelled));
        }
    }

    #[test]
    fn pin_errors() {
        for code in [SCARD_W_WRONG_CHV, SCARD_E_INVALID_CHV] {
            assert!(matches!(from_code("x", code), KeystoreError::WrongPin));
        }
        assert!(matches!(
            from_code("x", SCARD_W_CHV_BLOCKED),
            KeystoreError::PinLocked
        ));
        assert!(matches!(
            from_code("x", NTE_SILENT_CONTEXT),
            KeystoreError::PinRequired
        ));
    }

    #[test]
    fn card_gone_and_unsupported() {
        for code in [SCARD_W_REMOVED_CARD, SCARD_E_NO_SMARTCARD] {
            assert!(matches!(from_code("x", code), KeystoreError::TokenRemoved));
        }
        let KeystoreError::Unsupported(detail) = from_code("CryptCreateHash", NTE_BAD_ALGID) else {
            panic!("expected Unsupported");
        };
        assert!(detail.starts_with("CryptCreateHash: NTE_BAD_ALGID"));
    }

    #[test]
    fn anything_else_keeps_api_code_and_symbolic_name() {
        let KeystoreError::Native { api, code, message } =
            from_code("NCryptSignHash", NTE_BAD_KEYSET)
        else {
            panic!("expected a native error");
        };
        assert_eq!(api, "NCryptSignHash");
        assert_eq!(code, 0x8009_0016);
        assert!(message.starts_with("NTE_BAD_KEYSET"));
    }
}
