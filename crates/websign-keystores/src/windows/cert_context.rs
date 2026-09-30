//! Owned certificate contexts and their properties.

use std::ptr::NonNull;

use windows::Win32::Security::Cryptography::{
    CERT_CONTEXT, CERT_HASH_PROP_ID, CertFreeCertificateContext, CertGetCertificateContextProperty,
};

use super::thumbprint::Thumbprint;

/// One owned reference to a certificate context.
#[derive(Debug)]
pub struct CertContext(NonNull<CERT_CONTEXT>);

impl CertContext {
    pub(super) fn from_owned(context: *mut CERT_CONTEXT) -> Option<Self> {
        NonNull::new(context).map(Self)
    }

    pub fn as_ptr(&self) -> *const CERT_CONTEXT {
        self.0.as_ptr()
    }

    /// The DER encoding, borrowed from the context.
    pub fn der(&self) -> &[u8] {
        // SAFETY: the context stays alive while `self` does and its encoded
        // bytes never change.
        encoded(unsafe { self.0.as_ref() })
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

/// The DER bytes of a live certificate context, borrowed from it.
pub fn encoded(context: &CERT_CONTEXT) -> &[u8] {
    if context.pbCertEncoded.is_null() {
        return &[];
    }
    // SAFETY: a live context owns `cbCertEncoded` bytes at `pbCertEncoded`
    // for as long as the context itself, which the borrow ties us to.
    unsafe { std::slice::from_raw_parts(context.pbCertEncoded, context.cbCertEncoded as usize) }
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
