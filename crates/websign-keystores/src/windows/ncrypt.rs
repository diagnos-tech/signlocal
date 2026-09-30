//! Signing with a CNG key (`NCryptSignHash`).

use std::ffi::c_void;
use std::ptr;

use websign_core::{HashAlgorithm, SignatureAlgorithm};
use windows::Win32::Foundation::{HWND, NTE_BUFFER_TOO_SMALL};
use windows::Win32::Security::Cryptography::{
    BCRYPT_PKCS1_PADDING_INFO, BCRYPT_PSS_PADDING_INFO, BCRYPT_SHA256_ALGORITHM,
    BCRYPT_SHA384_ALGORITHM, BCRYPT_SHA512_ALGORITHM, NCRYPT_FLAGS, NCRYPT_PAD_PKCS1_FLAG,
    NCRYPT_PAD_PSS_FLAG, NCRYPT_SILENT_FLAG, NCRYPT_WINDOW_HANDLE_PROPERTY, NCryptSetProperty,
    NCryptSignHash,
};
use windows::core::PCWSTR;

use super::MAX_SIGNATURE_LEN;
use super::errors;
use super::handles::NcryptKey;
use crate::{KeystoreError, SignRequest};
use log::trace;

pub const API: &str = "NCryptSignHash";

/// Signs the digest. ECDSA comes back as raw `r || s`, RSA as the
/// big-endian signature block: both already in their final format. With
/// `silent`, a provider that would need UI fails instead of showing it.
pub fn sign(
    key: &NcryptKey,
    request: &SignRequest<'_>,
    owner: Option<HWND>,
    silent: bool,
) -> Result<Vec<u8>, KeystoreError> {
    if let Some(owner) = owner {
        set_owner(key, owner);
    }
    let algorithm = hash_name(request.hash);
    let pkcs1 = BCRYPT_PKCS1_PADDING_INFO {
        pszAlgId: algorithm,
    };
    let pss = BCRYPT_PSS_PADDING_INFO {
        pszAlgId: algorithm,
        // The salt length every PAdES/CMS verifier expects.
        cbSalt: request.hash.digest_len() as u32,
    };
    let (padding, mut flags): (Option<*const c_void>, NCRYPT_FLAGS) = match request.algorithm {
        SignatureAlgorithm::Ecdsa => (None, NCRYPT_FLAGS(0)),
        SignatureAlgorithm::RsaPkcs1v15 => {
            (Some(ptr::from_ref(&pkcs1).cast()), NCRYPT_PAD_PKCS1_FLAG)
        }
        SignatureAlgorithm::RsaPss => (Some(ptr::from_ref(&pss).cast()), NCRYPT_PAD_PSS_FLAG),
    };

    if silent {
        flags |= NCRYPT_SILENT_FLAG;
    }
    trace!(
        "NCryptSignHash({} {}, silent: {silent})",
        request.hash, request.algorithm
    );

    // One call with a buffer that fits any key, like .NET does: a size
    // query first would be a second round trip to the card, and some
    // middlewares authenticate per call.
    let mut signature = vec![0u8; MAX_SIGNATURE_LEN];
    let mut len = 0u32;
    // SAFETY: `padding` points to a structure on this frame whose algorithm
    // name is a static string; the wrapper passes digest and output slices
    // with their real lengths.
    let mut result = unsafe {
        NCryptSignHash(
            key.raw(),
            padding,
            request.digest,
            Some(&mut signature),
            &mut len,
            flags,
        )
    };
    if result
        .as_ref()
        .is_err_and(|error| error.code() == NTE_BUFFER_TOO_SMALL)
    {
        signature = vec![0u8; len as usize];
        // SAFETY: as above, with the size the provider asked for.
        result = unsafe {
            NCryptSignHash(
                key.raw(),
                padding,
                request.digest,
                Some(&mut signature),
                &mut len,
                flags,
            )
        };
    }
    result.map_err(|error| errors::native(API, &error))?;
    signature.truncate(len as usize);
    Ok(signature)
}

/// Makes `owner` the parent of the provider's PIN dialog, so it opens in
/// front instead of behind the browser.
fn set_owner(key: &NcryptKey, owner: HWND) {
    let value = (owner.0 as usize).to_ne_bytes();
    // SAFETY: valid key handle; the property value is the HWND itself.
    // Best effort: a provider without UI may reject the property, which only
    // matters for where a dialog would have opened.
    let _ = unsafe {
        NCryptSetProperty(
            key.raw().into(),
            NCRYPT_WINDOW_HANDLE_PROPERTY,
            &value,
            NCRYPT_FLAGS(0),
        )
    };
}

fn hash_name(hash: HashAlgorithm) -> PCWSTR {
    match hash {
        HashAlgorithm::Sha256 => BCRYPT_SHA256_ALGORITHM,
        HashAlgorithm::Sha384 => BCRYPT_SHA384_ALGORITHM,
        HashAlgorithm::Sha512 => BCRYPT_SHA512_ALGORITHM,
    }
}
