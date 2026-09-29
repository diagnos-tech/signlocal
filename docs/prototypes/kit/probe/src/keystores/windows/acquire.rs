//! Opening a certificate's private key with `CryptAcquireCertificatePrivateKey`,
//! the one call that reaches CNG providers, legacy CSPs and minidrivers alike.

use std::ffi::c_void;
use std::ptr;

use windows::Win32::Foundation::HWND;
use windows::Win32::Security::Cryptography::{
    CERT_KEY_SPEC, CERT_NCRYPT_KEY_SPEC, CRYPT_ACQUIRE_ALLOW_NCRYPT_KEY_FLAG,
    CRYPT_ACQUIRE_COMPARE_KEY_FLAG, CRYPT_ACQUIRE_FLAGS, CRYPT_ACQUIRE_ONLY_NCRYPT_KEY_FLAG,
    CRYPT_ACQUIRE_PREFER_NCRYPT_KEY_FLAG, CRYPT_ACQUIRE_WINDOW_HANDLE_FLAG,
    CryptAcquireCertificatePrivateKey, HCRYPTPROV_OR_NCRYPT_KEY_HANDLE, NCRYPT_KEY_HANDLE,
};
use windows::core::BOOL;

use super::capi::{self, CapiKey};
use super::errors;
use super::handles::{CryptProv, NcryptKey};
use super::key_info::KeyLocation;
use super::ncrypt;
use super::store::CertContext;
use crate::keystores::{KeystoreError, NcryptPreference, SignRequest};

/// A private key ready to sign, kept with its certificate.
#[derive(Debug)]
pub struct AcquiredKey {
    // Declared first so it is released before the certificate.
    key: Key,
    /// A handle Windows caches on the certificate lives only as long as the
    /// certificate does, so the certificate is kept alive with it.
    _cert: CertContext,
}

#[derive(Debug)]
enum Key {
    Ncrypt(NcryptKey),
    Capi(CapiKey),
}

impl AcquiredKey {
    /// Opens the key. Only the key's own provider may show UI here (e.g.
    /// "insert your card"), owned by `owner`; the PIN is asked when signing.
    pub fn open(
        cert: CertContext,
        preference: NcryptPreference,
        owner: Option<HWND>,
    ) -> Result<Self, KeystoreError> {
        // The comparison catches a key-provider entry that points to the
        // wrong container (e.g. after a token was re-issued).
        let mut flags = CRYPT_ACQUIRE_COMPARE_KEY_FLAG | preference_flag(preference);
        if owner.is_some() {
            flags |= CRYPT_ACQUIRE_WINDOW_HANDLE_FLAG;
        }
        let parameters = owner
            .as_ref()
            .map(|owner| ptr::from_ref(owner).cast::<c_void>());
        let mut handle = HCRYPTPROV_OR_NCRYPT_KEY_HANDLE::default();
        let mut key_spec = CERT_KEY_SPEC(0);
        let mut caller_frees = BOOL(0);
        // SAFETY: `cert` is a live context; the out-pointers are locals; with
        // CRYPT_ACQUIRE_WINDOW_HANDLE_FLAG, `parameters` points to an HWND
        // that outlives the call.
        unsafe {
            CryptAcquireCertificatePrivateKey(
                cert.as_ptr(),
                flags,
                parameters,
                &mut handle,
                Some(&mut key_spec),
                Some(&mut caller_frees),
            )
        }
        .map_err(|error| errors::native("CryptAcquireCertificatePrivateKey", &error))?;

        let owned = caller_frees.as_bool();
        let key = if key_spec == CERT_NCRYPT_KEY_SPEC {
            // SAFETY: CERT_NCRYPT_KEY_SPEC means the handle is a CNG key,
            // ours to free exactly when `caller_frees` says so.
            Key::Ncrypt(unsafe { NcryptKey::new(NCRYPT_KEY_HANDLE(handle.0), owned) })
        } else {
            // SAFETY: any other spec means a CAPI provider context, with the
            // same ownership rule.
            let context = unsafe { CryptProv::new(handle.0, owned) };
            Key::Capi(CapiKey::new(
                context,
                key_spec.0,
                KeyLocation::of(&cert).as_ref(),
            )?)
        };
        Ok(Self { key, _cert: cert })
    }

    /// Signs and names the native call that did it.
    pub fn sign(
        &self,
        request: &SignRequest<'_>,
        owner: Option<HWND>,
    ) -> Result<(Vec<u8>, &'static str), KeystoreError> {
        match &self.key {
            Key::Ncrypt(key) => ncrypt::sign(key, request, owner).map(|bytes| (bytes, ncrypt::API)),
            Key::Capi(key) => capi::sign(key, request, owner).map(|bytes| (bytes, key.api())),
        }
    }
}

fn preference_flag(preference: NcryptPreference) -> CRYPT_ACQUIRE_FLAGS {
    match preference {
        NcryptPreference::Allow => CRYPT_ACQUIRE_ALLOW_NCRYPT_KEY_FLAG,
        NcryptPreference::Prefer => CRYPT_ACQUIRE_PREFER_NCRYPT_KEY_FLAG,
        NcryptPreference::Only => CRYPT_ACQUIRE_ONLY_NCRYPT_KEY_FLAG,
    }
}
