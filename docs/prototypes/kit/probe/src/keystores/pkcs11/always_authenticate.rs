//! Keys with `CKA_ALWAYS_AUTHENTICATE`: the PIN is asked again for every
//! signature, as qualified signature keys of eIDAS cards (Cartao de Cidadao,
//! DNIe, Estonian ID card) require.
//!
//! PKCS#11 wants `C_SignInit`, then `C_Login(CKU_CONTEXT_SPECIFIC)`, then
//! `C_Sign`. `cryptoki` 0.12 only offers `C_Sign` fused with its own
//! `C_SignInit`, so the signature itself is a raw `C_Sign` from `cryptoki-sys`.

use std::path::Path;

use cryptoki::object::ObjectHandle;
use cryptoki::session::Session;
use secrecy::SecretString;

use super::errors::{self, Context};
use super::login;
use super::mechanism::SignPlan;
use crate::keystores::KeystoreError;

/// Signs `plan.input` with `key`, authenticating again in the middle.
pub fn sign(
    module: &Path,
    session: &Session,
    plan: &SignPlan,
    key: ObjectHandle,
    protected_path: bool,
    pin: Option<&SecretString>,
) -> Result<Vec<u8>, KeystoreError> {
    let mapper = errors::mapper(Context {
        without_pin: pin.is_none(),
    });
    session.sign_init(&plan.mechanism, key).map_err(mapper)?;
    login::log_in_for_signature(session, protected_path, pin)?;
    raw_sign(module, session, &plan.input)
}

/// Single-part `C_Sign` called on the module itself. The module is already
/// loaded and initialized by `cryptoki`; opening it again only resolves the
/// same exported symbols of the same library instance.
fn raw_sign(module: &Path, session: &Session, input: &[u8]) -> Result<Vec<u8>, KeystoreError> {
    let failed = |code: u64| KeystoreError::Native {
        api: "C_Sign",
        code: code as i64,
        message: "C_Sign after the context-specific login".to_owned(),
    };
    // SAFETY: `module` is the library `cryptoki` loaded and initialized, so
    // `dlopen` returns that same instance and `session.handle()` is a live
    // session of it; `input` and `signature` outlive both calls and the
    // lengths passed are their real lengths.
    unsafe {
        let library = cryptoki_sys::Pkcs11::new(module).map_err(|error| {
            KeystoreError::Other(format!("cannot reopen {}: {error}", module.display()))
        })?;
        let mut length: cryptoki_sys::CK_ULONG = 0;
        let rv = library.C_Sign(
            session.handle(),
            input.as_ptr().cast_mut(),
            input.len() as _,
            std::ptr::null_mut(),
            &mut length,
        );
        if rv != 0 {
            return Err(failed(rv as u64));
        }
        let mut signature = vec![0u8; length as usize];
        let rv = library.C_Sign(
            session.handle(),
            input.as_ptr().cast_mut(),
            input.len() as _,
            signature.as_mut_ptr(),
            &mut length,
        );
        if rv != 0 {
            return Err(failed(rv as u64));
        }
        signature.truncate(length as usize);
        Ok(signature)
    }
}
