//! ECDSA verification over a digest that is already computed.

use super::VerifyError;
use crate::ecdsa::Curve;

/// Verifies a raw `r || s` signature.
///
/// A public key that is not a valid point of its curve (off the curve, the
/// identity, wrong size) is `UnsupportedKey`, not `InvalidSignature`: no
/// signature could ever verify against it.
///
/// The digest is used as FIPS 186-5 prescribes for any hash size: a digest
/// longer than the curve order is truncated to its leftmost bytes and a
/// shorter one is used as is. So P-256 with SHA-384 and P-521 with SHA-256
/// are both valid combinations.
pub(super) fn verify(
    curve: Curve,
    point: &[u8],
    digest: &[u8],
    signature: &[u8],
) -> Result<(), VerifyError> {
    match curve {
        Curve::P256 => p256_verify(point, digest, signature),
        Curve::P384 => p384_verify(point, digest, signature),
        Curve::P521 => p521_verify(point, digest, signature),
    }
}

/// One verifier per curve crate; the crates share a shape but not a type.
macro_rules! curve_verifier {
    ($name:ident, $curve:ident) => {
        fn $name(point: &[u8], digest: &[u8], signature: &[u8]) -> Result<(), VerifyError> {
            use $curve::ecdsa::signature::hazmat::PrehashVerifier;
            use $curve::ecdsa::{Signature, VerifyingKey};

            // A point that is not on the curve means the certificate carries
            // a key nobody can use, which is not the signature's fault.
            let key =
                VerifyingKey::from_sec1_bytes(point).map_err(|_| VerifyError::UnsupportedKey)?;
            let signature =
                Signature::from_slice(signature).map_err(|_| VerifyError::InvalidSignature)?;
            key.verify_prehash(digest, &signature)
                .map_err(|_| VerifyError::InvalidSignature)
        }
    };
}

curve_verifier!(p256_verify, p256);
curve_verifier!(p384_verify, p384);
curve_verifier!(p521_verify, p521);
