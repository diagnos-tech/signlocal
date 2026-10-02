//! Issuer certificates from the Windows chain engine
//! (`CertGetCertificateChain`), so a caller building CAdES/PAdES gets the
//! intermediates the user's machine already knows.

use std::ptr::{self, NonNull};

use windows::Win32::Security::Cryptography::{
    CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL, CERT_CHAIN_CONTEXT, CERT_CHAIN_DISABLE_AIA,
    CERT_CHAIN_DISABLE_AUTH_ROOT_AUTO_UPDATE, CERT_CHAIN_PARA, CertFreeCertificateChain,
    CertGetCertificateChain,
};

use super::cert_context::{CertContext, encoded};
use log::trace;

/// More would be a malformed or hostile chain; real ones have two to four.
const MAX_ISSUERS: usize = 8;

/// Issuers of `cert`, nearest first, leaf excluded. Built only from what is
/// installed or cached: no AIA download, no root update, no revocation
/// check, because this runs while a signature request waits and must never
/// reach the network. Empty when the engine finds nothing.
pub fn issuers(cert: &CertContext) -> Vec<Vec<u8>> {
    let Some(chain) = ChainContext::build(cert) else {
        return Vec::new();
    };
    chain
        .first_chain_certificates()
        .into_iter()
        .skip(1)
        .take(MAX_ISSUERS)
        .collect()
}

/// A chain context, freed on drop.
struct ChainContext(NonNull<CERT_CHAIN_CONTEXT>);

impl ChainContext {
    fn build(cert: &CertContext) -> Option<Self> {
        let parameters = CERT_CHAIN_PARA {
            cbSize: size_of::<CERT_CHAIN_PARA>() as u32,
            ..Default::default()
        };
        let flags = CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL
            | CERT_CHAIN_DISABLE_AIA
            | CERT_CHAIN_DISABLE_AUTH_ROOT_AUTO_UPDATE;
        let mut context = ptr::null_mut();
        // SAFETY: `cert` is a live context and `parameters` a valid, sized
        // structure on this frame; the default engine and the current time
        // are requested with `None`; the chain written to `context` is ours
        // to free.
        let result = unsafe {
            CertGetCertificateChain(
                None,
                cert.as_ptr(),
                None,
                None,
                &parameters,
                flags,
                None,
                &mut context,
            )
        };
        if let Err(error) = result {
            trace!("CertGetCertificateChain failed: {:#x}", error.code().0);
            return None;
        }
        NonNull::new(context).map(Self)
    }

    /// DER of every element of the first simple chain, leaf first.
    fn first_chain_certificates(&self) -> Vec<Vec<u8>> {
        // SAFETY: the context is live until drop; every array below is
        // owned by it and sized by its count field.
        unsafe {
            let context = self.0.as_ref();
            if context.cChain == 0 || context.rgpChain.is_null() {
                return Vec::new();
            }
            let Some(simple) = (*context.rgpChain).as_ref() else {
                return Vec::new();
            };
            if simple.rgpElement.is_null() {
                return Vec::new();
            }
            std::slice::from_raw_parts(simple.rgpElement, simple.cElement as usize)
                .iter()
                .filter_map(|element| element.as_ref()?.pCertContext.as_ref())
                .map(|certificate| encoded(certificate).to_vec())
                .collect()
        }
    }
}

impl Drop for ChainContext {
    fn drop(&mut self) {
        // SAFETY: the chain came from CertGetCertificateChain and is freed
        // once; the certificate bytes copied out of it no longer borrow it.
        unsafe { CertFreeCertificateChain(self.0.as_ptr()) };
    }
}
