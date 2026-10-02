//! Checks on a signing request made before any key is opened, so a request
//! that can only fail never makes the user type a PIN first.

use websign_core::CertInfo;

use crate::{FoundKey, KeystoreError, SignRequest};

/// Rejects a digest of the wrong length (CNG would sign any length for
/// ECDSA) and an algorithm the certificate's key cannot do (e.g. ECDSA with
/// an RSA key, which some providers only reject after the PIN).
pub fn check(key: &FoundKey, request: &SignRequest<'_>) -> Result<(), KeystoreError> {
    request
        .hash
        .check_digest(request.digest)
        .map_err(|error| KeystoreError::Other(error.to_string()))?;
    // A certificate the reader cannot parse is left to the provider to judge.
    if let Ok(info) = CertInfo::from_der(&key.cert_der)
        && !info.key.supports(request.algorithm)
    {
        return Err(KeystoreError::Unsupported(format!(
            "{} with this certificate's key",
            request.algorithm
        )));
    }
    Ok(())
}
