//! `sign`: one signature with a PKCS#11 private key.
//!
//! The flow: find the certificate again (its slot may have moved), check that
//! the token can do the mechanism, log in unless the token is still unlocked
//! from an earlier signature (`docs/plan.md` D5), find the private key, sign.
//!
//! Every way out leaves the login in a known place: after a first successful
//! signature the session is parked in [`Sessions`] (the token stays unlocked
//! until `end_sessions`); after a failure that followed a fresh login the
//! session is logged out; a failure on an already unlocked token leaves the
//! parked login as it was.

use std::path::Path;
use std::time::Instant;

use cryptoki::context::Pkcs11;
use cryptoki::session::Session;

use super::errors::{self, Context};
use super::finder::{self, Located};
use super::key_checks;
use super::locator::Locator;
use super::mechanism::{self, SignPlan};
use super::sessions::Sessions;
use super::{always_authenticate, login};
use crate::{FoundKey, KeystoreError, SignRequest, Signature};
use log::trace;

/// The loaded module a signature goes through.
#[derive(Debug, Clone, Copy)]
pub struct Module<'a> {
    pub pkcs11: &'a Pkcs11,
    /// The library file, for the raw `C_Sign` of re-authenticating keys.
    pub path: &'a Path,
}

pub fn sign(
    module: Module<'_>,
    sessions: &mut Sessions,
    key: &FoundKey,
    request: &SignRequest<'_>,
) -> Result<Signature, KeystoreError> {
    let locator = Locator::parse(&key.locator)
        .ok_or_else(|| KeystoreError::Other("malformed PKCS#11 locator".to_owned()))?;
    let plan = mechanism::plan(request.hash, request.algorithm, request.digest)?;
    let curve = key_checks::expected_curve(request.algorithm, &key.cert_der)?;

    let located = finder::find_certificate(module.pkcs11, &locator, &key.cert_der)?;
    let token = module
        .pkcs11
        .get_token_info(located.slot)
        .map_err(errors::mapper(Context::default()))?;
    key_checks::require_mechanism(module.pkcs11, located.slot, plan.mechanism_type())?;

    let started = Instant::now();
    let unlocked = sessions.is_unlocked(located.slot);
    let protected_path = token.protected_authentication_path();
    if !unlocked {
        trace!("C_Login (protected path: {protected_path})");
        login::log_in(&located.session, &token, request.pin)?;
    }
    trace!("C_Sign({} {})", request.hash, request.algorithm);
    let signed = sign_logged_in(module.path, &located, &plan, request, protected_path);
    let elapsed = started.elapsed();

    let Located { slot, session, .. } = located;
    match &signed {
        Ok(_) if !unlocked && token.login_required() => sessions.park(slot, session),
        Err(_) if !unlocked => log_out(session),
        // Unlocked before: the parked session still holds the login, and
        // closing this one does not end it.
        _ => {}
    }

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

/// Best effort: a token that is gone is logged out already, and closing the
/// session (dropping it) ends the login when it was the token's last.
fn log_out(session: Session) {
    trace!("C_Logout after a failed signature");
    let _ = session.logout();
}

/// Finds the private key (visible now that the token is logged in) and
/// signs: single-part `C_Sign`, or the re-authenticating variant for keys
/// that demand it.
fn sign_logged_in(
    module: &Path,
    located: &Located,
    plan: &SignPlan,
    request: &SignRequest<'_>,
    protected_path: bool,
) -> Result<Vec<u8>, KeystoreError> {
    let key = finder::find_private_key(&located.session, &located.id)?;
    key_checks::require_key_type(key.key_type, request.algorithm)?;
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
