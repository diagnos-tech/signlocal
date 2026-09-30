//! `CurrentUser\MY` and the certificates in it.

use std::ptr::{self, NonNull};

use windows::Win32::Security::Cryptography::{
    CERT_CONTEXT, CERT_CONTROL_STORE_FLAGS, CERT_FIND_HASH, CERT_HASH_PROP_ID,
    CERT_OPEN_STORE_FLAGS, CERT_QUERY_ENCODING_TYPE, CERT_STORE_CTRL_AUTO_RESYNC,
    CERT_STORE_OPEN_EXISTING_FLAG, CERT_STORE_PROV_SYSTEM_W, CERT_STORE_READONLY_FLAG,
    CERT_SYSTEM_STORE_CURRENT_USER, CRYPT_INTEGER_BLOB, CertCloseStore, CertControlStore,
    CertDuplicateCertificateContext, CertEnumCertificatesInStore, CertFindCertificateInStore,
    CertFreeCertificateContext, CertGetCertificateContextProperty, CertOpenStore, HCERTSTORE,
    PKCS_7_ASN_ENCODING, X509_ASN_ENCODING,
};
use windows::core::w;

use super::errors;
use super::thumbprint::Thumbprint;
use crate::KeystoreError;

/// The current user's personal store, opened read-only: the probe must never
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

/// One owned reference to a certificate context.
#[derive(Debug)]
pub struct CertContext(NonNull<CERT_CONTEXT>);

impl CertContext {
    fn from_owned(context: *mut CERT_CONTEXT) -> Option<Self> {
        NonNull::new(context).map(Self)
    }

    pub fn as_ptr(&self) -> *const CERT_CONTEXT {
        self.0.as_ptr()
    }

    /// The DER encoding, borrowed from the context.
    pub fn der(&self) -> &[u8] {
        // SAFETY: the context stays alive while `self` does and its encoded
        // bytes never change.
        let context = unsafe { self.0.as_ref() };
        if context.pbCertEncoded.is_null() {
            return &[];
        }
        // SAFETY: as above; `cbCertEncoded` is the length of that buffer.
        unsafe { std::slice::from_raw_parts(context.pbCertEncoded, context.cbCertEncoded as usize) }
    }

    pub fn thumbprint(&self) -> Option<Thumbprint> {
        let property = self.property(CERT_HASH_PROP_ID)?;
        property.bytes().try_into().ok().map(Thumbprint)
    }

    /// A certificate property, or `None` when it is not set.
    pub fn property(&self, id: u32) -> Option<Property> {
        let mut len = 0u32;
        // SAFETY: size query: no output buffer, `len` receives the size.
        unsafe { CertGetCertificateContextProperty(self.as_ptr(), id, None, &mut len) }.ok()?;
        let mut words = vec![0u64; (len as usize).div_ceil(8)];
        // SAFETY: `words` has room for `len` bytes and the 8-byte alignment
        // that structured properties (with pointers) need.
        unsafe {
            CertGetCertificateContextProperty(
                self.as_ptr(),
                id,
                Some(words.as_mut_ptr().cast()),
                &mut len,
            )
        }
        .ok()?;
        Some(Property {
            words,
            len: len as usize,
        })
    }
}

impl Drop for CertContext {
    fn drop(&mut self) {
        // SAFETY: we own exactly one reference to this context.
        let _ = unsafe { CertFreeCertificateContext(Some(self.as_ptr())) };
    }
}

/// A property value in an 8-byte aligned buffer.
#[derive(Debug)]
pub struct Property {
    words: Vec<u64>,
    len: usize,
}

impl Property {
    pub fn bytes(&self) -> &[u8] {
        // SAFETY: `words` owns at least `len` initialized bytes (the second
        // call never reports more than the size it was given).
        unsafe { std::slice::from_raw_parts(self.words.as_ptr().cast(), self.len) }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.words.as_ptr().cast()
    }
}
