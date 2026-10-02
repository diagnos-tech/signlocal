//! Symbolic names of the status codes key providers return, so an error
//! report reads `NTE_BAD_KEYSET` (searchable in vendor documentation and
//! support tickets) instead of a bare number in the user's language.

use windows::Win32::Foundation::{
    CRYPT_E_NO_KEY_PROPERTY, ERROR_CANCELLED, NTE_BAD_ALGID, NTE_BAD_KEY, NTE_BAD_KEYSET,
    NTE_BAD_PROVIDER, NTE_BAD_SIGNATURE, NTE_BUFFER_TOO_SMALL, NTE_INVALID_HANDLE,
    NTE_INVALID_PARAMETER, NTE_NO_KEY, NTE_NOT_SUPPORTED, NTE_PERM, NTE_PROV_TYPE_NOT_DEF,
    NTE_SILENT_CONTEXT, NTE_USER_CANCELLED, SCARD_E_CARD_UNSUPPORTED, SCARD_E_INVALID_CHV,
    SCARD_E_NO_KEY_CONTAINER, SCARD_E_NO_READERS_AVAILABLE, SCARD_E_NO_SMARTCARD,
    SCARD_E_PIN_CACHE_EXPIRED, SCARD_E_READER_UNAVAILABLE, SCARD_E_TIMEOUT,
    SCARD_E_UNSUPPORTED_FEATURE, SCARD_W_CANCELLED_BY_USER, SCARD_W_CARD_NOT_AUTHENTICATED,
    SCARD_W_CHV_BLOCKED, SCARD_W_REMOVED_CARD, SCARD_W_SECURITY_VIOLATION, SCARD_W_UNPOWERED_CARD,
    SCARD_W_UNRESPONSIVE_CARD, SCARD_W_WRONG_CHV,
};
use windows::core::HRESULT;

/// The codes seen from CNG, CAPI and smart card minidrivers when opening
/// or using a key.
const NAMES: &[(HRESULT, &str)] = &[
    (CRYPT_E_NO_KEY_PROPERTY, "CRYPT_E_NO_KEY_PROPERTY"),
    (HRESULT::from_win32(ERROR_CANCELLED.0), "ERROR_CANCELLED"),
    (NTE_BAD_ALGID, "NTE_BAD_ALGID"),
    (NTE_BAD_KEY, "NTE_BAD_KEY"),
    (NTE_BAD_KEYSET, "NTE_BAD_KEYSET"),
    (NTE_BAD_PROVIDER, "NTE_BAD_PROVIDER"),
    (NTE_BAD_SIGNATURE, "NTE_BAD_SIGNATURE"),
    (NTE_BUFFER_TOO_SMALL, "NTE_BUFFER_TOO_SMALL"),
    (NTE_INVALID_HANDLE, "NTE_INVALID_HANDLE"),
    (NTE_INVALID_PARAMETER, "NTE_INVALID_PARAMETER"),
    (NTE_NO_KEY, "NTE_NO_KEY"),
    (NTE_NOT_SUPPORTED, "NTE_NOT_SUPPORTED"),
    (NTE_PERM, "NTE_PERM"),
    (NTE_PROV_TYPE_NOT_DEF, "NTE_PROV_TYPE_NOT_DEF"),
    (NTE_SILENT_CONTEXT, "NTE_SILENT_CONTEXT"),
    (NTE_USER_CANCELLED, "NTE_USER_CANCELLED"),
    (SCARD_E_CARD_UNSUPPORTED, "SCARD_E_CARD_UNSUPPORTED"),
    (SCARD_E_INVALID_CHV, "SCARD_E_INVALID_CHV"),
    (SCARD_E_NO_KEY_CONTAINER, "SCARD_E_NO_KEY_CONTAINER"),
    (SCARD_E_NO_READERS_AVAILABLE, "SCARD_E_NO_READERS_AVAILABLE"),
    (SCARD_E_NO_SMARTCARD, "SCARD_E_NO_SMARTCARD"),
    (SCARD_E_PIN_CACHE_EXPIRED, "SCARD_E_PIN_CACHE_EXPIRED"),
    (SCARD_E_READER_UNAVAILABLE, "SCARD_E_READER_UNAVAILABLE"),
    (SCARD_E_TIMEOUT, "SCARD_E_TIMEOUT"),
    (SCARD_E_UNSUPPORTED_FEATURE, "SCARD_E_UNSUPPORTED_FEATURE"),
    (SCARD_W_CANCELLED_BY_USER, "SCARD_W_CANCELLED_BY_USER"),
    (
        SCARD_W_CARD_NOT_AUTHENTICATED,
        "SCARD_W_CARD_NOT_AUTHENTICATED",
    ),
    (SCARD_W_CHV_BLOCKED, "SCARD_W_CHV_BLOCKED"),
    (SCARD_W_REMOVED_CARD, "SCARD_W_REMOVED_CARD"),
    (SCARD_W_SECURITY_VIOLATION, "SCARD_W_SECURITY_VIOLATION"),
    (SCARD_W_UNPOWERED_CARD, "SCARD_W_UNPOWERED_CARD"),
    (SCARD_W_UNRESPONSIVE_CARD, "SCARD_W_UNRESPONSIVE_CARD"),
    (SCARD_W_WRONG_CHV, "SCARD_W_WRONG_CHV"),
];

/// `NTE_BAD_KEYSET` for 0x80090016; `None` for codes outside the table.
pub fn symbolic_name(code: HRESULT) -> Option<&'static str> {
    NAMES
        .iter()
        .find(|(known, _)| *known == code)
        .map(|(_, name)| *name)
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::{NTE_BAD_KEYSET, SCARD_W_WRONG_CHV};
    use windows::core::HRESULT;

    use super::{NAMES, symbolic_name};

    #[test]
    fn names_known_codes() {
        assert_eq!(symbolic_name(NTE_BAD_KEYSET), Some("NTE_BAD_KEYSET"));
        assert_eq!(symbolic_name(SCARD_W_WRONG_CHV), Some("SCARD_W_WRONG_CHV"));
        assert_eq!(symbolic_name(HRESULT(0)), None);
    }

    #[test]
    fn every_code_is_listed_once() {
        for (index, (code, _)) in NAMES.iter().enumerate() {
            assert!(!NAMES[index + 1..].iter().any(|(other, _)| other == code));
        }
    }
}
