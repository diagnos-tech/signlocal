//! Signing with a legacy CAPI key (`CryptSignHash`): old token CSPs and
//! A1 certificates imported into Microsoft's software CSPs.

use std::ptr;

use probe_core::{HashAlgorithm, SignatureAlgorithm};
use windows::Win32::Foundation::{ERROR_MORE_DATA, HWND, NTE_BAD_ALGID};
use windows::Win32::Security::Cryptography::{
    ALG_ID, CALG_SHA_256, CALG_SHA_384, CALG_SHA_512, CRYPT_MACHINE_KEYSET, CRYPT_SILENT,
    CryptAcquireContextW, CryptSetHashParam, CryptSetProvParam, CryptSignHashW, HP_HASHVAL,
    MS_ENH_RSA_AES_PROV_W, PP_CLIENT_HWND, PROV_RSA_AES, PROV_RSA_FULL,
};
use windows::core::{HRESULT, HSTRING, PCWSTR};

use super::MAX_SIGNATURE_LEN;
use super::errors;
use super::handles::{CryptHash, CryptProv};
use super::key_info::KeyLocation;
use crate::keystores::{KeystoreError, SignRequest};
use crate::trace::trace;

pub const API: &str = "CryptSignHash";
/// [`API`] after reopening the container in the AES CSP.
pub const API_VIA_AES: &str = "CryptSignHash (PROV_RSA_AES)";

/// Microsoft's `PROV_RSA_FULL` software CSPs, which predate SHA-2. The
/// certificate import wizard puts PFX keys (A1 certificates) in them.
const SHA1_ONLY_CSPS: [&str; 3] = [
    "Microsoft Base Cryptographic Provider v1.0",
    "Microsoft Enhanced Cryptographic Provider v1.0",
    "Microsoft Strong Cryptographic Provider",
];

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

fn needs_aes_provider(location: &KeyLocation) -> bool {
    location.provider_type == PROV_RSA_FULL
        && SHA1_ONLY_CSPS
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&location.provider))
}

fn open_in_aes_provider(location: &KeyLocation, silent: bool) -> Result<CryptProv, KeystoreError> {
    let container = HSTRING::from(location.container.as_str());
    let mut flags = if location.machine_keyset {
        CRYPT_MACHINE_KEYSET.0
    } else {
        0
    };
    if silent {
        flags |= CRYPT_SILENT;
    }
    trace!("CryptAcquireContext(same container, Enhanced RSA and AES CSP, silent: {silent})");
    let mut handle = 0usize;
    // SAFETY: container and provider names are NUL-terminated and outlive
    // the call; `handle` is a live out-pointer.
    unsafe {
        CryptAcquireContextW(
            &mut handle,
            &container,
            MS_ENH_RSA_AES_PROV_W,
            PROV_RSA_AES,
            flags,
        )
    }
    .map_err(|error| errors::native("CryptAcquireContext", &error))?;
    // SAFETY: just acquired; released only by the returned value.
    Ok(unsafe { CryptProv::new(handle, true) })
}

/// Signs with RSASSA-PKCS1-v1_5, the only scheme CAPI has.
pub fn sign(
    key: &CapiKey,
    request: &SignRequest<'_>,
    owner: Option<HWND>,
) -> Result<Vec<u8>, KeystoreError> {
    if request.algorithm != SignatureAlgorithm::RsaPkcs1v15 {
        return Err(KeystoreError::Unsupported(format!(
            "legacy CAPI key: {} needs CNG (try --ncrypt prefer)",
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
                "CSP \"{}\" cannot sign {} hashes (try --ncrypt prefer)",
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

#[cfg(test)]
mod tests {
    use super::needs_aes_provider;
    use crate::keystores::windows::key_info::KeyLocation;

    fn location(provider: &str, provider_type: u32) -> KeyLocation {
        KeyLocation {
            provider: provider.to_owned(),
            provider_type,
            container: String::new(),
            key_spec: 1,
            machine_keyset: false,
        }
    }

    #[test]
    fn only_microsoft_sha1_software_csps_are_reopened() {
        assert!(needs_aes_provider(&location(
            "Microsoft Enhanced Cryptographic Provider v1.0",
            1
        )));
        assert!(needs_aes_provider(&location(
            "microsoft base cryptographic provider v1.0",
            1
        )));
        assert!(!needs_aes_provider(&location(
            "Microsoft Enhanced RSA and AES Cryptographic Provider",
            24
        )));
        assert!(!needs_aes_provider(&location(
            "Microsoft Base Smart Card Crypto Provider",
            1
        )));
        assert!(!needs_aes_provider(&location(
            "eToken Base Cryptographic Provider",
            1
        )));
    }
}
