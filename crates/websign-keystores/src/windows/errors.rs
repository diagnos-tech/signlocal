//! Windows status codes turned into [`KeystoreError`], so the app can tell a
//! cancelled PIN dialog or a wrong PIN apart from a real failure.

use windows::Win32::Foundation::{
    ERROR_CANCELLED, NTE_USER_CANCELLED, SCARD_E_INVALID_CHV, SCARD_W_CANCELLED_BY_USER,
    SCARD_W_CHV_BLOCKED, SCARD_W_WRONG_CHV,
};
use windows::core::HRESULT;

use crate::KeystoreError;

/// Maps the error of the native call `api`.
pub fn native(api: &'static str, error: &windows::core::Error) -> KeystoreError {
    from_code(api, error.code())
}

/// Maps a status code. Smart card minidrivers report through `SCARD_*`,
/// CNG providers through `NTE_*`, and some CSPs through plain Win32 errors.
pub fn from_code(api: &'static str, code: HRESULT) -> KeystoreError {
    const CANCELLED: [HRESULT; 3] = [
        SCARD_W_CANCELLED_BY_USER,
        NTE_USER_CANCELLED,
        HRESULT::from_win32(ERROR_CANCELLED.0),
    ];
    if CANCELLED.contains(&code) {
        KeystoreError::Cancelled
    } else if code == SCARD_W_WRONG_CHV || code == SCARD_E_INVALID_CHV {
        KeystoreError::WrongPin
    } else if code == SCARD_W_CHV_BLOCKED {
        KeystoreError::PinLocked
    } else {
        KeystoreError::Native {
            api,
            // Unsigned, so it prints as the familiar 0x8010006b.
            code: i64::from(code.0 as u32),
            message: code.message(),
        }
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::{
        ERROR_CANCELLED, NTE_BAD_ALGID, NTE_USER_CANCELLED, SCARD_E_INVALID_CHV,
        SCARD_W_CANCELLED_BY_USER, SCARD_W_CHV_BLOCKED, SCARD_W_WRONG_CHV,
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
        assert!(matches!(
            from_code("x", SCARD_W_WRONG_CHV),
            KeystoreError::WrongPin
        ));
        assert!(matches!(
            from_code("x", SCARD_E_INVALID_CHV),
            KeystoreError::WrongPin
        ));
        assert!(matches!(
            from_code("x", SCARD_W_CHV_BLOCKED),
            KeystoreError::PinLocked
        ));
    }

    #[test]
    fn anything_else_keeps_api_code_and_message() {
        let KeystoreError::Native { api, code, message } =
            from_code("CryptCreateHash", NTE_BAD_ALGID)
        else {
            panic!("expected a native error");
        };
        assert_eq!(api, "CryptCreateHash");
        assert_eq!(code, 0x8009_0008);
        assert!(!message.is_empty());
    }
}
