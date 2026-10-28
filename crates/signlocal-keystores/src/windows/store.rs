//! `CurrentUser\MY` and the certificates in it.

use std::ptr;

use windows::Win32::Security::Cryptography::{
    CERT_CONTEXT, CERT_CONTROL_STORE_FLAGS, CERT_FIND_HASH, CERT_OPEN_STORE_FLAGS,
    CERT_QUERY_ENCODING_TYPE, CERT_STORE_CTRL_AUTO_RESYNC, CERT_STORE_OPEN_EXISTING_FLAG,
    CERT_STORE_PROV_SYSTEM_W, CERT_STORE_READONLY_FLAG, CERT_SYSTEM_STORE_CURRENT_USER,
    CRYPT_INTEGER_BLOB, CertCloseStore, CertControlStore, CertDuplicateCertificateContext,
    CertEnumCertificatesInStore, CertFindCertificateInStore, CertFreeCertificateContext,
    CertOpenStore, HCERTSTORE, PKCS_7_ASN_ENCODING, X509_ASN_ENCODING,
};
use windows::core::w;

use super::cert_context::CertContext;
use super::errors;
use super::thumbprint::Thumbprint;
use crate::KeystoreError;

/// The current user's personal store, opened read-only: the app must never
/// change what the user has installed.
#[derive(Debug)]
pub struct CertStore(HCERTSTORE);

impl CertStore {
    pub fn open_current_user_my() -> Result<Self, KeystoreError> {
        let flags = CERT_OPEN_STORE_FLAGS(CERT_SYSTEM_STORE_CURRENT_USER)
            | CERT_STORE_OPEN_EXISTING_FLAG
            | CERT_STORE_READONLY_FLAG;
        // SAFETY: for CERT_STORE_PROV_SYSTEM_W the parameter is the store
        // name, a static NUL-terminated UTF-16 string.
        let handle = unsafe {
            CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                CERT_QUERY_ENCODING_TYPE(0),
                None,
                flags,
                Some(w!("MY").as_ptr().cast()),
            )
        }
        .map_err(|error| errors::native("CertOpenStore", &error))?;
        let store = CertStore(handle);
        // The certificate propagation service adds and removes token
        // certificates while the app runs; with auto-resync every enumeration
        // sees the current contents. Best effort: without it the store is a
        // snapshot, which is still right for a single run.
        // SAFETY: valid store handle; this control type takes no parameter.
        let _ = unsafe {
            CertControlStore(
                store.0,
                CERT_CONTROL_STORE_FLAGS(0),
                CERT_STORE_CTRL_AUTO_RESYNC,
                None,
            )
        };
        Ok(store)
    }

    pub fn certificates(&self) -> Certificates<'_> {
        Certificates {
            store: self,
            previous: ptr::null(),
            finished: false,
        }
    }

    /// The certificate with this SHA-1 thumbprint, if it is still installed.
    pub fn find(&self, thumbprint: &Thumbprint) -> Option<CertContext> {
        let blob = CRYPT_INTEGER_BLOB {
            cbData: thumbprint.0.len() as u32,
            pbData: thumbprint.0.as_ptr().cast_mut(),
        };
        // SAFETY: `blob` describes 20 live bytes that CertFind only reads;
        // the returned context, if any, is ours to free.
        let found = unsafe {
            CertFindCertificateInStore(
                self.0,
                X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
                0,
                CERT_FIND_HASH,
                Some(ptr::from_ref(&blob).cast()),
                None,
            )
        };
        CertContext::from_owned(found)
    }
}

impl Drop for CertStore {
    fn drop(&mut self) {
        // SAFETY: the handle is ours. Without CERT_CLOSE_STORE_FORCE_FLAG,
        // contexts that are still referenced keep the store's memory alive.
        let _ = unsafe { CertCloseStore(Some(self.0), 0) };
    }
}

/// Iterates a store, handing out one owned reference per certificate.
#[derive(Debug)]
pub struct Certificates<'a> {
    store: &'a CertStore,
    /// The last context CertEnum returned; the next call frees it.
    previous: *const CERT_CONTEXT,
    finished: bool,
}

impl Iterator for Certificates<'_> {
    type Item = CertContext;

    fn next(&mut self) -> Option<CertContext> {
        if self.finished {
            return None;
        }
        // SAFETY: `previous` is null or the context the last call returned;
        // CertEnum releases it and returns the next one (or null at the end).
        let next = unsafe { CertEnumCertificatesInStore(self.store.0, Some(self.previous)) };
        self.previous = next;
        if next.is_null() {
            self.finished = true;
            return None;
        }
        // SAFETY: `next` is live; duplicating adds the reference the item
        // owns, independent of the one the enumeration holds.
        CertContext::from_owned(unsafe { CertDuplicateCertificateContext(Some(next)) })
    }
}

impl Drop for Certificates<'_> {
    fn drop(&mut self) {
        if !self.previous.is_null() {
            // SAFETY: an enumeration stopped early still owns `previous`.
            let _ = unsafe { CertFreeCertificateContext(Some(self.previous)) };
        }
    }
}
