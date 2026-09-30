//! Signing a digest with an identity's private key.
//!
//! The OS owns the PIN: a token driver or the keychain shows its own dialog
//! from inside `SecKeyCreateSignature` when the key needs it.

use std::time::Instant;

use core_foundation::base::TCFType;
use security_framework::identity::SecIdentity;
use security_framework::key::{Algorithm, SecKey};
use security_framework_sys::key::{SecKeyIsAlgorithmSupported, kSecKeyOperationTypeSign};
use websign_core::{CertInfo, Curve, PublicKeyKind, SignatureAlgorithm, ecdsa};

use super::{algorithm, errors, status};
use crate::{KeystoreError, SignRequest, Signature};

const API: &str = "SecKeyCreateSignature";

/// Signs `request.digest` and returns the signature in the SDK's format:
/// the RSA block as is, ECDSA converted from DER to raw `r || s`.
pub fn sign(
    identity: &SecIdentity,
    cert_der: &[u8],
    request: &SignRequest<'_>,
) -> Result<Signature, KeystoreError> {
    // The ECDSA "Digest" algorithms accept any length, so the rule that only
    // digests of the declared size are signed is enforced here too.
    request
        .hash
        .check_digest(request.digest)
        .map_err(|error| KeystoreError::Other(error.to_string()))?;
    let curve = match request.algorithm {
        SignatureAlgorithm::Ecdsa => Some(ec_curve(cert_der)?),
        SignatureAlgorithm::RsaPkcs1v15 | SignatureAlgorithm::RsaPss => None,
    };
    let key = identity
        .private_key()
        .map_err(|error| status::from_status("SecIdentityCopyPrivateKey", error.code()))?;
    let sec_algorithm = algorithm::for_digest(request.hash, request.algorithm);
    if !supports(&key, sec_algorithm) {
        return Err(KeystoreError::Unsupported(format!(
            "{} with {} (SecKeyIsAlgorithmSupported)",
            request.algorithm, request.hash
        )));
    }

    let started = Instant::now();
    let output = key
        .create_signature(sec_algorithm, request.digest)
        .map_err(|error| errors::from_cf_error(API, &error))?;
    let elapsed = started.elapsed();

    let bytes = match curve {
        Some(curve) => ecdsa::der_to_raw(&output, curve).map_err(|error| {
            KeystoreError::Other(format!(
                "{API} returned an unusable ECDSA signature: {error}"
            ))
        })?,
        None => output,
    };
    Ok(Signature {
        bytes,
        api: API,
        elapsed,
    })
}

/// Asks the key (for token keys, the token driver) before signing, so an
/// unsupported combination is reported as such rather than as a failure.
fn supports(key: &SecKey, algorithm: Algorithm) -> bool {
    // SAFETY: `key` keeps the SecKeyRef alive for the call, and the algorithm
    // is an immutable CFString constant exported by Security.framework.
    unsafe {
        SecKeyIsAlgorithmSupported(
            key.as_concrete_TypeRef(),
            kSecKeyOperationTypeSign,
            algorithm.into(),
        ) != 0
    }
}

/// The curve of an EC certificate, which sizes the raw signature.
fn ec_curve(cert_der: &[u8]) -> Result<Curve, KeystoreError> {
    let info =
        CertInfo::from_der(cert_der).map_err(|error| KeystoreError::Other(error.to_string()))?;
    match info.key {
        PublicKeyKind::Ec { curve } => Ok(curve),
        other => Err(KeystoreError::Unsupported(format!(
            "ECDSA needs an EC key on a named curve, found {other:?}"
        ))),
    }
}
