//! Checks a raw signature against a certificate's public key.
//!
//! The kit uses it to prove that what an OS or token returned is a valid
//! signature in exactly the format the SDK promises.

mod ec_verify;
#[cfg(test)]
mod fixtures;
mod rsa_verify;
#[cfg(test)]
mod tests;

use crate::algorithm::SignatureAlgorithm;
use crate::cert::{self, CertError, CertInfo, PublicKeyKind};
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
    let certificate = cert::decode_certificate(cert_der)?;
    let info = CertInfo::from_certificate(cert_der, &certificate)?;
    hash.check_digest(digest)?;

    let spki = certificate.tbs_certificate().subject_public_key_info();
    match info.key {
        PublicKeyKind::Unsupported { .. } => Err(VerifyError::UnsupportedKey),
        ref key if !key.supports(algorithm) => Err(VerifyError::KeyMismatch(algorithm)),
        PublicKeyKind::Rsa { .. } => {
            let key = cert::rsa_components(spki)?;
            rsa_verify::verify(key, hash, algorithm, digest, signature)
        }
        PublicKeyKind::Ec { curve } => {
            let point = cert::ec_point(spki).ok_or(VerifyError::UnsupportedKey)?;
            ec_verify::verify(curve, point, digest, signature)
        }
    }
}
