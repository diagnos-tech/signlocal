//! `C_Login`: the only place a PIN leaves its `SecretString`.
//!
//! The PIN goes straight from the caller's `SecretString` into the call:
//! no copy is made in this module, and nothing here prints or logs it.

use cryptoki::error::{Error, RvError};
use cryptoki::session::{Session, UserType};
use cryptoki::slot::TokenInfo;
use secrecy::SecretString;

use super::errors::{self, Context};
use crate::KeystoreError;

/// `C_Login(CKU_USER)`, unless the token needs none.
///
/// A locked PIN is reported before trying: a wrong attempt on a locked or
/// nearly locked token is never worth making.
pub fn log_in(
    session: &Session,
    token: &TokenInfo,
    pin: Option<&SecretString>,
) -> Result<(), KeystoreError> {
    if !token.login_required() {
        return Ok(());
    }
    if token.user_pin_locked() {
        return Err(KeystoreError::PinLocked);
    }
    let pin = pin_for(token.protected_authentication_path(), pin)?;
    match session.login(UserType::User, pin) {
        Ok(()) | Err(Error::Pkcs11(RvError::UserAlreadyLoggedIn, _)) => Ok(()),
        Err(error) => Err(errors::map(
            error,
            Context {
                without_pin: pin.is_none(),
            },
        )),
    }
}

/// `C_Login(CKU_CONTEXT_SPECIFIC)`: the extra authentication of a key that
/// demands it for every signature. Only valid between `C_SignInit` and the signature.
pub fn log_in_for_signature(
    session: &Session,
    protected_path: bool,
    pin: Option<&SecretString>,
) -> Result<(), KeystoreError> {
    let pin = pin_for(protected_path, pin)?;
    session
        .login(UserType::ContextSpecific, pin)
        .map_err(errors::mapper(Context {
            without_pin: pin.is_none(),
        }))
}

/// `None` when the reader's PIN pad collects the PIN (`C_Login` is then called
/// with a null PIN), otherwise the app's PIN, which must exist.
fn pin_for(
    protected_path: bool,
    pin: Option<&SecretString>,
) -> Result<Option<&SecretString>, KeystoreError> {
    if protected_path {
        return Ok(None);
    }
    pin.map(Some).ok_or(KeystoreError::PinRequired)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pin_pad_means_no_pin_is_passed_even_when_the_app_has_one() {
        let pin = SecretString::from("1234".to_owned());
        assert!(pin_for(true, Some(&pin)).unwrap().is_none());
        assert!(pin_for(true, None).unwrap().is_none());
    }

    #[test]
    fn without_a_pin_pad_the_apps_pin_is_required() {
        assert!(matches!(
            pin_for(false, None),
            Err(KeystoreError::PinRequired)
        ));
        let pin = SecretString::from("1234".to_owned());
        assert!(pin_for(false, Some(&pin)).unwrap().is_some());
    }
}
