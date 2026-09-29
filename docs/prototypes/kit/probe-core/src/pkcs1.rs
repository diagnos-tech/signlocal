//! PKCS#1 v1.5 `DigestInfo`, for signers that take an already-hashed input.

use crate::hash::{DigestLengthError, HashAlgorithm};

/// DER `DigestInfo` (RFC 8017 §9.2): the hash's fixed prefix followed by `digest`.
///
/// `CKM_RSA_PKCS` and other "raw" RSA signers pad whatever they receive, so
/// the caller must wrap the digest itself; hashing mechanisms such as
/// `CKM_SHA256_RSA_PKCS` would hash the digest a second time.
pub fn digest_info(hash: HashAlgorithm, digest: &[u8]) -> Result<Vec<u8>, DigestLengthError> {
    todo!()
}
