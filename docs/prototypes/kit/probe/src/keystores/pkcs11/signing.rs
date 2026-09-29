//! `sign`: one signature with a PKCS#11 private key.
//!
//! The flow is: find the certificate again (its slot may have moved), check
//! that the token can do the mechanism, log in with the PIN, find the private
//! key, sign, log out (whatever happened after the login). Nothing is cached
//! between signatures: the PIN is used for one `C_Login` and the session is
//! closed afterwards, so the token asks for it again next time.

use std::time::Instant;

use cryptoki::context::Pkcs11;
use cryptoki::error::{Error, RvError};
use cryptoki::mechanism::MechanismType;
use cryptoki::object::KeyType;
use cryptoki::slot::Slot;
use probe_core::{CertInfo, PublicKeyKind, SignatureAlgorithm};

use super::errors::{self, Context};
use super::finder::{self, Located, PrivateKey};
use super::locator::Locator;
use super::mechanism::{self, SignPlan};
use super::{always_authenticate, login};
use crate::keystores::{FoundKey, KeystoreError, SignRequest, Signature};
use crate::trace::trace;

pub fn sign(
    pkcs11: &Pkcs11,
    module: &std::path::Path,
    key: &FoundKey,
    request: &SignRequest<'_>,
) -> Result<Signature, KeystoreError> {
    let locator = Locator::parse(&key.locator).ok_or_else(|| {
        KeystoreError::Other(format!("malformed PKCS#11 locator {:?}", key.locator))
    })?;
    let plan = mechanism::plan(request.hash, request.algorithm, request.digest)?;
    let curve = expected_curve(request.algorithm, &key.cert_der)?;

    let located = finder::find_certificate(pkcs11, &locator, &key.cert_der)?;
    let token = pkcs11
        .get_token_info(located.slot)
        .map_err(errors::mapper(Context::default()))?;
    require_mechanism(pkcs11, located.slot, plan.mechanism_type())?;

    let started = Instant::now();
    let protected_path = token.protected_authentication_path();
    trace!("C_Login (protected path: {protected_path})");
    login::log_in(&located.session, &token, request.pin)?;
    trace!("C_Sign({} {})", request.hash, request.algorithm);
    let signed = sign_logged_in(module, &located, &plan, request, protected_path);
    let elapsed = started.elapsed();
    // On failure too, so the login never outlives this call. Best effort:
    // closing the last session of the token logs out as well.
    let _ = located.session.logout();
    let raw = signed?;
    let bytes = match curve {
        Some(curve) => mechanism::ecdsa_signature(raw, curve)?,
        None => raw,
    };
    Ok(Signature {
        bytes,
        api: "C_Sign",
        elapsed,
    })
}

/// For ECDSA, the curve from the certificate (needed to validate the raw
/// signature's size); `None` for RSA.
fn expected_curve(
    algorithm: SignatureAlgorithm,
    cert_der: &[u8],
) -> Result<Option<probe_core::Curve>, KeystoreError> {
    if algorithm != SignatureAlgorithm::Ecdsa {
        return Ok(None);
    }
    let info =
        CertInfo::from_der(cert_der).map_err(|error| KeystoreError::Other(error.to_string()))?;
    match info.key {
        PublicKeyKind::Ec { curve } => Ok(Some(curve)),
        other => Err(KeystoreError::Unsupported(format!(
            "ECDSA needs an EC key, the certificate has {other:?}"
        ))),
    }
}

/// `Unsupported` when the token says it cannot sign with the mechanism. A
/// token that cannot even describe it is given the benefit of the doubt: some
/// modules implement mechanisms they do not report.
fn require_mechanism(
    pkcs11: &Pkcs11,
    slot: Slot,
    mechanism: MechanismType,
) -> Result<(), KeystoreError> {
    match pkcs11.get_mechanism_info(slot, mechanism) {
        Ok(info) if info.sign() => Ok(()),
        Ok(_) | Err(Error::Pkcs11(RvError::MechanismInvalid, _)) => Err(
            KeystoreError::Unsupported(format!("the token cannot sign with {mechanism}")),
        ),
        Err(_) => Ok(()),
    }
}

fn check_key_type(key: &PrivateKey, algorithm: SignatureAlgorithm) -> Result<(), KeystoreError> {
    let expected = if algorithm.is_rsa() {
        KeyType::RSA
    } else {
        KeyType::EC
    };
    match key.key_type {
        Some(actual) if actual != expected => Err(KeystoreError::Unsupported(format!(
            "{algorithm} needs a{} key",
            if algorithm.is_rsa() { "n RSA" } else { "n EC" }
        ))),
        _ => Ok(()),
    }
}

/// Finds the private key (visible now that the session is logged in) and
/// signs: single-part `C_Sign`, or the re-authenticating variant for keys
/// that demand it.
fn sign_logged_in(
    module: &std::path::Path,
    located: &Located,
    plan: &SignPlan,
    request: &SignRequest<'_>,
    protected_path: bool,
) -> Result<Vec<u8>, KeystoreError> {
    let key = finder::find_private_key(&located.session, &located.id)?;
    check_key_type(&key, request.algorithm)?;
    if key.always_authenticate {
        return always_authenticate::sign(
            module,
            &located.session,
            plan,
            key.handle,
            protected_path,
            request.pin,
        );
    }
    located
        .session
        .sign(&plan.mechanism, key.handle, &plan.input)
        .map_err(errors::mapper(Context {
            without_pin: request.pin.is_none(),
        }))
}
