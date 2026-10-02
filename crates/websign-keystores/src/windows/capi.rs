//! Signing with a legacy CAPI key (`CryptSignHash`): old token CSPs and
//! A1 certificates imported into Microsoft's software CSPs.

use std::ptr;

use websign_core::{HashAlgorithm, SignatureAlgorithm};
use windows::Win32::Foundation::{ERROR_MORE_DATA, HWND, NTE_BAD_ALGID};
use windows::Win32::Security::Cryptography::{
    ALG_ID, CALG_SHA_256, CALG_SHA_384, CALG_SHA_512, CryptSetHashParam, CryptSetProvParam,
    CryptSignHashW, HP_HASHVAL, PP_CLIENT_HWND,
};
use windows::core::{HRESULT, PCWSTR};

use super::MAX_SIGNATURE_LEN;
use super::aes_reopen::{needs_aes_provider, open_in_aes_provider};
use super::errors;
use super::handles::{CryptHash, CryptProv};
use super::key_info::KeyLocation;
use crate::{KeystoreError, SignRequest};
use log::trace;

pub const API: &str = "CryptSignHash";
/// [`API`] after reopening the container in the AES CSP.
pub const API_VIA_AES: &str = "CryptSignHash (PROV_RSA_AES)";

/// A CAPI key ready to sign.
#[derive(Debug)]
pub struct CapiKey {
    context: CryptProv,
    key_spec: u32,
    provider: String,
    api: &'static str,
}

impl CapiKey {
    /// Wraps a context from `CryptAcquireCertificatePrivateKey`. Keys in a
    /// SHA-1-only Microsoft CSP are reopened in the Enhanced RSA and AES CSP,
    /// which reads the same key containers (what .NET does for SHA-2),
    /// with `CRYPT_SILENT` when `silent`.
    pub fn new(
        context: CryptProv,
        key_spec: u32,
        location: Option<&KeyLocation>,
        silent: bool,
    ) -> Result<Self, KeystoreError> {
        let provider = location.map_or_else(String::new, |at| at.provider.clone());
        match location.filter(|at| needs_aes_provider(at)) {
            Some(at) => Ok(Self {
                context: open_in_aes_provider(at, silent)?,
                key_spec,
                provider,
                api: API_VIA_AES,
            }),
            None => Ok(Self {
                context,
                key_spec,
                provider,
                api: API,
            }),
        }
    }

    pub fn api(&self) -> &'static str {
        self.api
    }
}

/// Signs with RSASSA-PKCS1-v1_5, the only scheme CAPI has.
pub fn sign(
    key: &CapiKey,
    request: &SignRequest<'_>,
    owner: Option<HWND>,
) -> Result<Vec<u8>, KeystoreError> {
    if request.algorithm != SignatureAlgorithm::RsaPkcs1v15 {
        return Err(KeystoreError::Unsupported(format!(
            "{} needs CNG, and this key's CSP only signs through legacy CAPI",
            request.algorithm
        )));
    }
    // HP_HASHVAL reads as many bytes as the hash produces, whatever we pass.
    request
        .hash
        .check_digest(request.digest)
        .map_err(|error| KeystoreError::Other(error.to_string()))?;
    if let Some(owner) = owner {
        set_owner(owner);
    }
    let hash = CryptHash::create(&key.context, algorithm_id(request.hash)).map_err(|error| {
        if error.code() == NTE_BAD_ALGID {
            KeystoreError::Unsupported(format!(
                "CSP \"{}\" cannot sign {} hashes",
                key.provider, request.hash
            ))
        } else {
            errors::native("CryptCreateHash", &error)
        }
    })?;
    // SAFETY: valid hash; `digest` has exactly the hash length (checked above).
    unsafe { CryptSetHashParam(hash.raw(), HP_HASHVAL, request.digest.as_ptr(), 0) }
        .map_err(|error| errors::native("CryptSetHashParam", &error))?;

    trace!(
        "CryptSignHash({} {}, key spec {})",
        request.hash, request.algorithm, key.key_spec
    );
    let mut signature = vec![0u8; MAX_SIGNATURE_LEN];
    let mut len = signature.len() as u32;
    // SAFETY: `signature` has `len` writable bytes; the description must be null.
    let mut result = unsafe {
        CryptSignHashW(
            hash.raw(),
            key.key_spec,
            PCWSTR::null(),
            0,
            Some(signature.as_mut_ptr()),
            &mut len,
        )
    };
    if result
        .as_ref()
        .is_err_and(|error| error.code() == HRESULT::from_win32(ERROR_MORE_DATA.0))
    {
        signature = vec![0u8; len as usize];
        // SAFETY: as above, with the size the CSP asked for.
        result = unsafe {
            CryptSignHashW(
                hash.raw(),
                key.key_spec,
                PCWSTR::null(),
                0,
                Some(signature.as_mut_ptr()),
                &mut len,
            )
        };
    }
    result.map_err(|error| errors::native(API, &error))?;
    signature.truncate(len as usize);
    // CAPI returns the signature little-endian.
    signature.reverse();
    Ok(signature)
}

/// Makes `owner` the parent of CSP dialogs. A null provider context applies
/// it to every CAPI context in the process, the documented way to do it.
fn set_owner(owner: HWND) {
    // SAFETY: PP_CLIENT_HWND reads one HWND through the pointer, and `owner`
    // outlives the call.
    let _ = unsafe { CryptSetProvParam(0, PP_CLIENT_HWND, ptr::from_ref(&owner).cast(), 0) };
}

fn algorithm_id(hash: HashAlgorithm) -> ALG_ID {
    match hash {
        HashAlgorithm::Sha256 => CALG_SHA_256,
        HashAlgorithm::Sha384 => CALG_SHA_384,
        HashAlgorithm::Sha512 => CALG_SHA_512,
    }
}
