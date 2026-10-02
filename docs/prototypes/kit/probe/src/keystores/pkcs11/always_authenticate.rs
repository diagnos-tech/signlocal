//! Keys with `CKA_ALWAYS_AUTHENTICATE`: the PIN is asked again for every
//! signature, as qualified signature keys of eIDAS cards (Cartao de Cidadao,
//! DNIe, Estonian ID card) require.
//!
//! PKCS#11 wants `C_SignInit`, then `C_Login(CKU_CONTEXT_SPECIFIC)`, then
//! `C_Sign`. `cryptoki` 0.12 only offers `C_Sign` fused with its own
//! `C_SignInit`, so the signature itself is a raw `C_Sign` taken from the
//! module's function list.

use std::path::Path;
use std::ptr;

use cryptoki::context::Function;
use cryptoki::error::{Error, Rv};
use cryptoki::object::ObjectHandle;
use cryptoki::session::Session;
use cryptoki_sys::{CK_FUNCTION_LIST, CK_RV, CK_ULONG, CKR_BUFFER_TOO_SMALL};
use secrecy::SecretString;

use super::errors::{self, Context};
use super::login;
use super::mechanism::SignPlan;
use crate::keystores::KeystoreError;

/// Room for an RSA-16384 signature, larger than any key a token holds.
const SIGNATURE_CAPACITY: usize = 2048;

/// Signs `plan.input` with `key`, authenticating again in the middle.
pub fn sign(
    module: &Path,
    session: &Session,
    plan: &SignPlan,
    key: ObjectHandle,
    protected_path: bool,
    pin: Option<&SecretString>,
) -> Result<Vec<u8>, KeystoreError> {
    let context = Context {
        without_pin: pin.is_none(),
    };
    session
        .sign_init(&plan.mechanism, key)
        .map_err(errors::mapper(context))?;
    login::log_in_for_signature(session, protected_path, pin)?;
    raw_sign(module, session, &plan.input).map_err(errors::mapper(context))
}

/// Single-part `C_Sign` on `session`, called through `C_GetFunctionList`:
/// that is the one entry point PKCS#11 requires a module to export, so a
/// module that does not export `C_Sign` by name still works.
///
/// One call with a buffer that fits any key, and a second one only when the
/// module answers `CKR_BUFFER_TOO_SMALL` (the operation stays active then).
/// A size query first would be an extra `C_Sign` after the context-specific
/// login, and some modules let that login cover a single call.
fn raw_sign(module: &Path, session: &Session, input: &[u8]) -> Result<Vec<u8>, Error> {
    // SAFETY: `module` is the library `cryptoki` already loaded and
    // initialized. Loading it again returns that same instance without
    // running its initializers; dropping `library` at the end of this
    // function only releases the extra reference taken here.
    let library = unsafe { cryptoki_sys::Pkcs11::new(module) }.map_err(Error::LibraryLoading)?;
    let get_function_list = library
        .C_GetFunctionList
        .as_ref()
        .map_err(|_| Error::MissingSymbol("C_GetFunctionList"))?;
    let mut table: *mut CK_FUNCTION_LIST = ptr::null_mut();
    // SAFETY: `table` is a live out-pointer, which the module sets to its
    // own function table.
    Rv::from(unsafe { get_function_list(&mut table) }).into_result(Function::GetFunctionList)?;
    // SAFETY: the table is null-checked by `as_ref` and belongs to the
    // module, which stays loaded for the life of the process.
    let c_sign = unsafe { table.as_ref() }
        .and_then(|table| table.C_Sign)
        .ok_or(Error::NullFunctionPointer)?;

    let input_len = CK_ULONG::try_from(input.len())?;
    let call = |signature: &mut [u8]| -> Result<(CK_RV, usize), Error> {
        let mut length = CK_ULONG::try_from(signature.len())?;
        // SAFETY: `session.handle()` is a live session of this module with a
        // signing operation initialized on it; `input` and `signature` are
        // valid for the lengths passed, and the module only reads `input`
        // (the pointer is mutable in the C prototype only).
        let rv = unsafe {
            c_sign(
                session.handle(),
                input.as_ptr().cast_mut(),
                input_len,
                signature.as_mut_ptr(),
                &mut length,
            )
        };
        Ok((rv, usize::try_from(length)?))
    };

    let mut signature = vec![0u8; SIGNATURE_CAPACITY];
    let (mut rv, mut length) = call(&mut signature)?;
    if rv == CKR_BUFFER_TOO_SMALL {
        signature = vec![0u8; length];
        (rv, length) = call(&mut signature)?;
    }
    Rv::from(rv).into_result(Function::Sign)?;
    signature.truncate(length);
    Ok(signature)
}
