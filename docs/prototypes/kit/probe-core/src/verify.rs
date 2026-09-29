//! Checks a raw signature against a certificate's public key.
//!
//! The kit uses it to prove that what an OS or token returned is a valid
//! signature in exactly the format the SDK promises.

use crate::algorithm::SignatureAlgorithm;
use crate::cert::CertError;
use crate::hash::{DigestLengthError, HashAlgorithm};

/// Why a signature was not accepted, in the order the checks run.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VerifyError {
    #[error(transparent)]
    Certificate(#[from] CertError),
    #[error(transparent)]
    Digest(#[from] DigestLengthError),
    #[error("unsupported public key")]
    UnsupportedKey,
    #[error("{0} cannot be used with this key")]
    KeyMismatch(SignatureAlgorithm),
    #[error("invalid signature")]
    InvalidSignature,
}

/// Verifies `signature` over `digest` with the key in `cert_der`.
///
/// RSA signatures are modulus-sized blocks; ECDSA signatures are raw
/// `r || s`. PSS uses MGF1 with `hash` and a salt as long as the digest.
pub fn verify(
    cert_der: &[u8],
    hash: HashAlgorithm,
    algorithm: SignatureAlgorithm,
    digest: &[u8],
    signature: &[u8],
) -> Result<(), VerifyError> {
    todo!()
}
