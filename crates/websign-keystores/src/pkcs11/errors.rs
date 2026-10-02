//! Translation of `cryptoki` errors into [`KeystoreError`].
//!
//! The UI reacts to the typed variants (wrong PIN, locked, cancelled), so the
//! mapping decides what the person sees. Everything else keeps its raw
//! `CKR_*` code for support.

use cryptoki::context::Function;
use cryptoki::error::{Error, RvError};

use super::ckr;
use crate::KeystoreError;

/// What the caller knows about the failed call, when it changes the meaning
/// of the code.
#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    /// The call was made without a PIN (protected authentication path, or the
    /// caller had none), so "not logged in" means "a PIN is required".
    pub without_pin: bool,
}

/// Maps a `cryptoki` error to the keystore error the callers understand.
pub fn map(error: Error, context: Context) -> KeystoreError {
    match error {
        Error::Pkcs11(rv, function) => map_return_value(rv, function, context),
        // Token information the module fills with something that is not what
        // PKCS#11 says (a clock flag with a blank clock, for one).
        Error::ParseInt(_) | Error::Utf8(_) | Error::InvalidValue => {
            KeystoreError::Other(format!("the module returned malformed data ({error})"))
        }
        other => KeystoreError::Other(other.to_string()),
    }
}

/// Convenience for `.map_err(errors::mapper(...))` in call chains.
pub fn mapper(context: Context) -> impl Fn(Error) -> KeystoreError {
    move |error| map(error, context)
}

fn map_return_value(rv: RvError, function: Function, context: Context) -> KeystoreError {
    match rv {
        RvError::PinIncorrect => KeystoreError::WrongPin,
        // A PIN of the wrong length can never be right, and the token has not
        // counted it as an attempt; from the person's side it is a wrong PIN.
        RvError::PinInvalid | RvError::PinLenRange => KeystoreError::WrongPin,
        RvError::PinLocked => KeystoreError::PinLocked,
        // CKR_FUNCTION_REJECTED is what modules that ask for the PIN in their
        // own dialog answer when the person presses "Cancel".
        RvError::FunctionCanceled | RvError::Cancel | RvError::FunctionRejected => {
            KeystoreError::Cancelled
        }
        RvError::UserNotLoggedIn if context.without_pin => KeystoreError::PinRequired,
        // The session died with the token: the card was pulled, or the
        // reader reset it (`SPEC.md` §3.1).
        RvError::DeviceRemoved
        | RvError::TokenNotPresent
        | RvError::SessionHandleInvalid
        | RvError::SessionClosed => KeystoreError::TokenRemoved,
        RvError::MechanismInvalid => {
            KeystoreError::Unsupported("the token does not offer this signing mechanism".to_owned())
        }
        _ => native(rv, function),
    }
}

fn native(rv: RvError, function: Function) -> KeystoreError {
    let (code, name) = ckr::describe(rv);
    let message = match ckr::hint(rv) {
        Some(hint) => format!("{name} ({hint})"),
        None => name.to_owned(),
    };
    KeystoreError::Native {
        api: api_name(function),
        code,
        message,
    }
}

/// The PKCS#11 function name, for messages such as "C_Login failed".
pub fn api_name(function: Function) -> &'static str {
    match function {
        Function::Initialize => "C_Initialize",
        Function::GetInfo => "C_GetInfo",
        Function::GetSlotList => "C_GetSlotList",
        Function::GetSlotInfo => "C_GetSlotInfo",
        Function::GetTokenInfo => "C_GetTokenInfo",
        Function::GetMechanismInfo => "C_GetMechanismInfo",
        Function::OpenSession => "C_OpenSession",
        Function::CloseSession => "C_CloseSession",
        Function::Login => "C_Login",
        Function::Logout => "C_Logout",
        Function::GetAttributeValue => "C_GetAttributeValue",
        Function::FindObjectsInit => "C_FindObjectsInit",
        Function::FindObjects => "C_FindObjects",
        Function::FindObjectsFinal => "C_FindObjectsFinal",
        Function::SignInit => "C_SignInit",
        Function::Sign => "C_Sign",
        Function::SignUpdate => "C_SignUpdate",
        Function::SignFinal => "C_SignFinal",
        _ => "PKCS#11",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_rv(rv: RvError, function: Function, without_pin: bool) -> KeystoreError {
        map(Error::Pkcs11(rv, function), Context { without_pin })
    }

    #[test]
    fn pin_problems_get_their_own_variants() {
        let login = Function::Login;
        assert!(matches!(
            map_rv(RvError::PinIncorrect, login, false),
            KeystoreError::WrongPin
        ));
        assert!(matches!(
            map_rv(RvError::PinLenRange, login, false),
            KeystoreError::WrongPin
        ));
        assert!(matches!(
            map_rv(RvError::PinLocked, login, false),
            KeystoreError::PinLocked
        ));
    }

    #[test]
    fn cancelling_in_a_module_dialog_is_a_cancellation() {
        for rv in [
            RvError::FunctionCanceled,
            RvError::FunctionRejected,
            RvError::Cancel,
        ] {
            assert!(matches!(
                map_rv(rv, Function::Sign, false),
                KeystoreError::Cancelled
            ));
        }
    }

    #[test]
    fn not_logged_in_only_means_pin_required_without_a_pin() {
        let without = map_rv(RvError::UserNotLoggedIn, Function::Sign, true);
        assert!(matches!(without, KeystoreError::PinRequired));
        let with = map_rv(RvError::UserNotLoggedIn, Function::Sign, false);
        assert!(matches!(with, KeystoreError::Native { code: 0x101, .. }));
    }

    #[test]
    fn a_vanished_token_or_session_means_the_token_was_removed() {
        for rv in [
            RvError::DeviceRemoved,
            RvError::TokenNotPresent,
            RvError::SessionHandleInvalid,
            RvError::SessionClosed,
        ] {
            assert!(matches!(
                map_rv(rv, Function::Sign, false),
                KeystoreError::TokenRemoved
            ));
        }
    }

    #[test]
    fn everything_else_keeps_the_raw_code_and_the_function() {
        match map_rv(RvError::DeviceError, Function::Sign, false) {
            KeystoreError::Native { api, code, message } => {
                assert_eq!(api, "C_Sign");
                assert_eq!(code, 0x30);
                assert_eq!(message, "CKR_DEVICE_ERROR");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn hints_are_appended_to_the_code_name() {
        let error = map_rv(RvError::PinExpired, Function::Login, false);
        assert_eq!(
            error.to_string(),
            "C_Login failed with 0xa3: CKR_PIN_EXPIRED (the PIN must be changed with the vendor's tool)"
        );
    }

    #[test]
    fn errors_that_are_not_return_values_become_other() {
        assert!(matches!(
            map(Error::InvalidValue, Context::default()),
            KeystoreError::Other(_)
        ));
    }
}
