//! Native key handles, each released exactly once by its owner.

use std::marker::PhantomData;

use windows::Win32::Security::Cryptography::{
    ALG_ID, CryptCreateHash, CryptDestroyHash, CryptReleaseContext, NCRYPT_KEY_HANDLE,
    NCryptFreeObject,
};

/// A CAPI provider context (`HCRYPTPROV`).
#[derive(Debug)]
pub struct CryptProv {
    handle: usize,
    /// False when Windows keeps the handle cached on the certificate.
    owned: bool,
}

impl CryptProv {
    /// # Safety
    /// `handle` must be a valid provider context. When `owned`, nothing else
    /// may release it.
    pub unsafe fn new(handle: usize, owned: bool) -> Self {
        Self { handle, owned }
    }

    pub fn raw(&self) -> usize {
        self.handle
    }
}

impl Drop for CryptProv {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: we own the context and release it once.
            let _ = unsafe { CryptReleaseContext(self.handle, 0) };
        }
    }
}

/// A CAPI hash object, which must not outlive its provider context.
#[derive(Debug)]
pub struct CryptHash<'a> {
    handle: usize,
    provider: PhantomData<&'a CryptProv>,
}

impl<'a> CryptHash<'a> {
    pub fn create(provider: &'a CryptProv, algorithm: ALG_ID) -> windows::core::Result<Self> {
        let mut handle = 0;
        // SAFETY: valid provider context; `handle` is a live out-pointer.
        unsafe { CryptCreateHash(provider.raw(), algorithm, 0, 0, &mut handle) }?;
        Ok(Self {
            handle,
            provider: PhantomData,
        })
    }

    pub fn raw(&self) -> usize {
        self.handle
    }
}

impl Drop for CryptHash<'_> {
    fn drop(&mut self) {
        // SAFETY: we created the hash and destroy it once, before its
        // provider context is released (enforced by the lifetime).
        let _ = unsafe { CryptDestroyHash(self.handle) };
    }
}

/// A CNG key handle.
#[derive(Debug)]
pub struct NcryptKey {
    handle: NCRYPT_KEY_HANDLE,
    /// False when Windows keeps the handle cached on the certificate.
    owned: bool,
}

impl NcryptKey {
    /// # Safety
    /// `handle` must be a valid key handle. When `owned`, nothing else may
    /// free it.
    pub unsafe fn new(handle: NCRYPT_KEY_HANDLE, owned: bool) -> Self {
        Self { handle, owned }
    }

    pub fn raw(&self) -> NCRYPT_KEY_HANDLE {
        self.handle
    }
}

impl Drop for NcryptKey {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: we own the handle and free it once.
            let _ = unsafe { NCryptFreeObject(self.handle.into()) };
        }
    }
}
